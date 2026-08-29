use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use russh::client::{self, Config};
use russh::keys::known_hosts::learn_known_hosts_path;
use russh::keys::{check_known_hosts_path, decode_secret_key, PrivateKeyWithHashAlg, PublicKey};
use russh::{ChannelMsg, Disconnect};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

use super::{OpenTunnel, SshTunnelDriver, SshTunnelRequest, TransportErrorReporter};
use crate::{
    MqttEndpoint, MqttError, MqttResult, SecretBytes, SshAuth, SshFailureKind, SshHostKeyPolicy,
    SshTunnelOptions,
};

pub(crate) struct RusshTunnelDriver;

#[async_trait]
impl SshTunnelDriver for RusshTunnelDriver {
    async fn open(&self, request: SshTunnelRequest) -> MqttResult<Box<dyn OpenTunnel>> {
        Ok(Box::new(RusshTunnel::open(request).await?))
    }
}

struct RusshTunnel {
    local_endpoint: MqttEndpoint,
    session: Arc<client::Handle<Client>>,
    accept_task: JoinHandle<()>,
}

impl RusshTunnel {
    async fn open(request: SshTunnelRequest) -> MqttResult<Self> {
        validate_options(&request.options)?;
        let mut session = connect_session(&request.options).await?;
        authenticate(&mut session, &request.options).await?;
        let session = Arc::new(session);
        let (listener, local_endpoint) = bind_local(request.options.local_bind_port).await?;
        let accept_task = spawn_accept_loop(
            listener,
            Arc::clone(&session),
            request.remote_endpoint,
            request.error_reporter,
        );

        Ok(Self {
            local_endpoint,
            session,
            accept_task,
        })
    }
}

#[async_trait]
impl OpenTunnel for RusshTunnel {
    fn local_endpoint(&self) -> MqttEndpoint {
        self.local_endpoint.clone()
    }

    async fn close(&mut self) -> MqttResult<()> {
        self.accept_task.abort();
        self.session
            .disconnect(Disconnect::ByApplication, "CorreoMQTT tunnel closed", "en")
            .await
            .map_err(|error| MqttError::ssh_failure(SshFailureKind::Teardown, error.to_string()))
    }
}

impl Drop for RusshTunnel {
    fn drop(&mut self) {
        self.accept_task.abort();
    }
}

#[derive(Clone)]
struct Client {
    host_key_policy: SshHostKeyPolicy,
    host: String,
    port: u16,
}

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        match &self.host_key_policy {
            SshHostKeyPolicy::AcceptAnyInsecure => Ok(true),
            SshHostKeyPolicy::TrustOnFirstUse { known_hosts_path } => {
                match check_known_hosts_path(
                    &self.host,
                    self.port,
                    server_public_key,
                    known_hosts_path,
                ) {
                    Ok(true) => Ok(true),
                    // Unknown host (or file not created yet): trust on first
                    // use and pin the key for future connections. Any other
                    // error (changed key, unreadable/corrupt pin file) rejects
                    // rather than silently re-pinning.
                    Ok(false) => Ok(learn_known_hosts_path(
                        &self.host,
                        self.port,
                        server_public_key,
                        known_hosts_path,
                    )
                    .is_ok()),
                    Err(russh::keys::Error::IO(error))
                        if error.kind() == std::io::ErrorKind::NotFound =>
                    {
                        Ok(learn_known_hosts_path(
                            &self.host,
                            self.port,
                            server_public_key,
                            known_hosts_path,
                        )
                        .is_ok())
                    }
                    Err(_) => Ok(false),
                }
            }
        }
    }
}

fn validate_options(options: &SshTunnelOptions) -> MqttResult<()> {
    if options.host.trim().is_empty() {
        return Err(MqttError::ssh_failure(
            SshFailureKind::Connect,
            "SSH host is required",
        ));
    }
    if options.port == 0 {
        return Err(MqttError::ssh_failure(
            SshFailureKind::Connect,
            "SSH port must be greater than zero",
        ));
    }
    if options.username.trim().is_empty() {
        return Err(MqttError::ssh_failure(
            SshFailureKind::Auth,
            "SSH username is required",
        ));
    }
    Ok(())
}

async fn connect_session(options: &SshTunnelOptions) -> MqttResult<client::Handle<Client>> {
    let config = Arc::new(Config {
        nodelay: true,
        ..Default::default()
    });
    client::connect(
        config,
        (options.host.as_str(), options.port),
        Client {
            host_key_policy: options.host_key_policy.clone(),
            host: options.host.clone(),
            port: options.port,
        },
    )
    .await
    .map_err(|error| MqttError::ssh_failure(SshFailureKind::Connect, error.to_string()))
}

async fn authenticate(
    session: &mut client::Handle<Client>,
    options: &SshTunnelOptions,
) -> MqttResult<()> {
    let result = match &options.auth {
        SshAuth::Password(password) => {
            session
                .authenticate_password(
                    options.username.clone(),
                    password.expose_secret().to_owned(),
                )
                .await
        }
        SshAuth::PrivateKey {
            path,
            private_key,
            passphrase,
        } => {
            let key = load_private_key(path.as_deref(), private_key.as_ref(), passphrase.as_ref())
                .await?;
            let hash_alg = session
                .best_supported_rsa_hash()
                .await
                .map_err(|error| MqttError::ssh_failure(SshFailureKind::Auth, error.to_string()))?
                .flatten();
            session
                .authenticate_publickey(
                    options.username.clone(),
                    PrivateKeyWithHashAlg::new(Arc::new(key), hash_alg),
                )
                .await
        }
    }
    .map_err(|error| MqttError::ssh_failure(SshFailureKind::Auth, error.to_string()))?;

    if result.success() {
        Ok(())
    } else {
        Err(MqttError::ssh_failure(
            SshFailureKind::Auth,
            "SSH authentication was rejected",
        ))
    }
}

async fn load_private_key(
    path: Option<&str>,
    private_key: Option<&SecretBytes>,
    passphrase: Option<&crate::SecretString>,
) -> MqttResult<russh::keys::PrivateKey> {
    let secret = match (private_key, path) {
        (Some(private_key), _) => {
            String::from_utf8(private_key.expose_secret().to_vec()).map_err(|_| {
                MqttError::ssh_failure(SshFailureKind::Auth, "SSH private key is not UTF-8 PEM")
            })?
        }
        (None, Some(path)) => tokio::fs::read_to_string(path)
            .await
            .map_err(|error| MqttError::ssh_failure(SshFailureKind::Auth, error.to_string()))?,
        (None, None) => {
            return Err(MqttError::ssh_failure(
                SshFailureKind::Auth,
                "SSH private key material is required",
            ));
        }
    };

    decode_secret_key(&secret, passphrase.map(crate::SecretString::expose_secret))
        .map_err(|error| MqttError::ssh_failure(SshFailureKind::Auth, error.to_string()))
}

async fn bind_local(local_bind_port: Option<u16>) -> MqttResult<(TcpListener, MqttEndpoint)> {
    let port = local_bind_port.unwrap_or(0);
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|error| MqttError::ssh_failure(SshFailureKind::Bind, error.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|error| MqttError::ssh_failure(SshFailureKind::Bind, error.to_string()))?
        .port();
    Ok((
        listener,
        MqttEndpoint {
            host: "127.0.0.1".to_owned(),
            port,
        },
    ))
}

fn spawn_accept_loop(
    listener: TcpListener,
    session: Arc<client::Handle<Client>>,
    remote_endpoint: MqttEndpoint,
    error_reporter: Option<TransportErrorReporter>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok((stream, originator)) = listener.accept().await {
            let session = Arc::clone(&session);
            let remote_endpoint = remote_endpoint.clone();
            let error_reporter = error_reporter.clone();
            tokio::spawn(async move {
                if let Err(error) = relay_stream(stream, originator, session, remote_endpoint).await
                {
                    if let Some(report_error) = error_reporter {
                        report_error(error);
                    }
                }
            });
        }
    })
}

async fn relay_stream(
    mut stream: TcpStream,
    originator: SocketAddr,
    session: Arc<client::Handle<Client>>,
    remote_endpoint: MqttEndpoint,
) -> MqttResult<()> {
    let mut channel = session
        .channel_open_direct_tcpip(
            remote_endpoint.host,
            u32::from(remote_endpoint.port),
            originator.ip().to_string(),
            u32::from(originator.port()),
        )
        .await
        .map_err(|error| {
            MqttError::ssh_failure(SshFailureKind::RemoteConnect, error.to_string())
        })?;

    let mut buffer = vec![0; 16 * 1024];
    let mut stream_closed = false;
    loop {
        tokio::select! {
            read = stream.read(&mut buffer), if !stream_closed => {
                match read {
                    Ok(0) => {
                        stream_closed = true;
                        channel.eof().await.map_err(map_remote_connect)?;
                    }
                    Ok(bytes_read) => {
                        channel.data(&buffer[..bytes_read]).await.map_err(map_remote_connect)?;
                    }
                    Err(error) => return Err(MqttError::ssh_failure(
                        SshFailureKind::RemoteConnect,
                        error.to_string(),
                    )),
                }
            }
            message = channel.wait() => {
                match message {
                    Some(ChannelMsg::Data { data }) => {
                        stream.write_all(&data).await.map_err(|error| {
                            MqttError::ssh_failure(SshFailureKind::RemoteConnect, error.to_string())
                        })?;
                    }
                    Some(ChannelMsg::Eof | ChannelMsg::Close) | None => break,
                    Some(ChannelMsg::WindowAdjusted { .. }) => {}
                    Some(_) => {}
                }
            }
        }
    }
    Ok(())
}

fn map_remote_connect(error: russh::Error) -> MqttError {
    MqttError::ssh_failure(SshFailureKind::RemoteConnect, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use russh::client::Handler;
    use russh::keys::ssh_key::public::{Ed25519PublicKey, KeyData};

    fn server_key(byte: u8) -> PublicKey {
        PublicKey::new(KeyData::Ed25519(Ed25519PublicKey([byte; 32])), "")
    }

    #[tokio::test]
    async fn tofu_pins_first_key_and_rejects_changed_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut client = Client {
            host_key_policy: SshHostKeyPolicy::TrustOnFirstUse {
                known_hosts_path: dir.path().join("known_hosts"),
            },
            host: "broker.example".to_owned(),
            port: 22,
        };

        assert!(client
            .check_server_key(&server_key(7))
            .await
            .expect("first use"));
        assert!(client
            .check_server_key(&server_key(7))
            .await
            .expect("same key"));
        assert!(!client
            .check_server_key(&server_key(9))
            .await
            .expect("changed key"));
    }
}
