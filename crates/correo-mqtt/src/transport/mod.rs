mod ssh;
pub(crate) mod tls;

use std::sync::Arc;

use async_trait::async_trait;

use crate::{MqttConnectionOptions, MqttEndpoint, MqttError, MqttResult};

pub(crate) use ssh::RusshTunnelDriver;

pub(crate) struct PreparedTransport {
    pub(crate) endpoint: MqttEndpoint,
    tunnel: Option<Box<dyn OpenTunnel>>,
}

impl PreparedTransport {
    pub(crate) async fn open_with_reporter(
        options: &MqttConnectionOptions,
        error_reporter: Option<TransportErrorReporter>,
    ) -> MqttResult<Self> {
        Self::open_with_driver(options, &RusshTunnelDriver, error_reporter).await
    }

    pub(crate) async fn open_with_driver(
        options: &MqttConnectionOptions,
        driver: &dyn SshTunnelDriver,
        error_reporter: Option<TransportErrorReporter>,
    ) -> MqttResult<Self> {
        tls::validate(&options.tls, options.ssh_tunnel.is_some())?;

        let Some(ssh_options) = &options.ssh_tunnel else {
            return Ok(Self {
                endpoint: options.endpoint.clone(),
                tunnel: None,
            });
        };

        let request = SshTunnelRequest {
            options: ssh_options.clone(),
            remote_endpoint: options.endpoint.clone(),
            error_reporter,
        };
        let tunnel = driver.open(request).await?;
        let endpoint = tunnel.local_endpoint();
        Ok(Self {
            endpoint,
            tunnel: Some(tunnel),
        })
    }

    pub(crate) async fn close(&mut self) -> MqttResult<()> {
        if let Some(tunnel) = &mut self.tunnel {
            tunnel.close().await?;
        }
        self.tunnel = None;
        Ok(())
    }
}

pub(crate) type TransportErrorReporter = Arc<dyn Fn(MqttError) + Send + Sync>;

#[derive(Clone)]
pub(crate) struct SshTunnelRequest {
    pub(crate) options: crate::SshTunnelOptions,
    pub(crate) remote_endpoint: MqttEndpoint,
    pub(crate) error_reporter: Option<TransportErrorReporter>,
}

#[async_trait]
pub(crate) trait SshTunnelDriver: Send + Sync {
    async fn open(&self, request: SshTunnelRequest) -> MqttResult<Box<dyn OpenTunnel>>;
}

#[async_trait]
pub(crate) trait OpenTunnel: Send + Sync {
    fn local_endpoint(&self) -> MqttEndpoint;

    async fn close(&mut self) -> MqttResult<()>;
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::{
        ConnectionId, MqttConnectionOptions, MqttError, SecretString, SshAuth, SshFailureKind,
        SshHostKeyPolicy, SshTunnelOptions,
    };

    #[tokio::test]
    async fn ssh_driver_rewrites_endpoint_and_closes_tunnel() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let driver = FakeDriver {
            state: Arc::clone(&state),
            fail_open: None,
            fail_close: None,
        };
        let mut prepared = PreparedTransport::open_with_driver(&options(), &driver, None)
            .await
            .expect("prepared");

        assert_eq!(prepared.endpoint.host, "127.0.0.1");
        assert_eq!(prepared.endpoint.port, 21883);

        prepared.close().await.expect("closed");
        let state = state.lock().expect("state");
        assert_eq!(state.open_count, 1);
        assert_eq!(state.close_count, 1);
        assert_eq!(state.last_remote.as_ref().expect("remote").port, 1883);
    }

    #[tokio::test]
    async fn ssh_driver_preserves_typed_failures() {
        let driver = FakeDriver {
            state: Arc::new(Mutex::new(FakeState::default())),
            fail_open: Some(MqttError::ssh_failure(
                SshFailureKind::Auth,
                "password rejected",
            )),
            fail_close: None,
        };
        let error = match PreparedTransport::open_with_driver(&options(), &driver, None).await {
            Ok(_) => panic!("expected auth failure"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            MqttError::Ssh {
                failure: SshFailureKind::Auth,
                ..
            }
        ));
    }

    fn options() -> MqttConnectionOptions {
        let mut options = MqttConnectionOptions::new(
            ConnectionId::new(),
            "ssh",
            MqttEndpoint::new("broker.example", 1883).expect("endpoint"),
        );
        options.ssh_tunnel = Some(SshTunnelOptions {
            host: "jump.example".to_owned(),
            port: 22,
            username: "ssh-user".to_owned(),
            auth: SshAuth::Password(SecretString::new("synthetic-password")),
            host_key_policy: SshHostKeyPolicy::AcceptAnyInsecure,
            local_bind_port: None,
        });
        options
    }

    #[derive(Default)]
    struct FakeState {
        open_count: usize,
        close_count: usize,
        last_remote: Option<MqttEndpoint>,
    }

    struct FakeDriver {
        state: Arc<Mutex<FakeState>>,
        fail_open: Option<MqttError>,
        fail_close: Option<MqttError>,
    }

    #[async_trait]
    impl SshTunnelDriver for FakeDriver {
        async fn open(&self, request: SshTunnelRequest) -> MqttResult<Box<dyn OpenTunnel>> {
            if let Some(error) = &self.fail_open {
                return Err(error.clone());
            }
            let mut state = self.state.lock().expect("state");
            state.open_count += 1;
            state.last_remote = Some(request.remote_endpoint);
            Ok(Box::new(FakeTunnel {
                state: Arc::clone(&self.state),
                fail_close: self.fail_close.clone(),
            }))
        }
    }

    struct FakeTunnel {
        state: Arc<Mutex<FakeState>>,
        fail_close: Option<MqttError>,
    }

    #[async_trait]
    impl OpenTunnel for FakeTunnel {
        fn local_endpoint(&self) -> MqttEndpoint {
            MqttEndpoint {
                host: "127.0.0.1".to_owned(),
                port: 21883,
            }
        }

        async fn close(&mut self) -> MqttResult<()> {
            if let Some(error) = &self.fail_close {
                return Err(error.clone());
            }
            self.state.lock().expect("state").close_count += 1;
            Ok(())
        }
    }
}
