use std::collections::HashMap;
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use correo_core::BuiltInBrokerProcessConfig;
use rumqttd::{Broker, Config, ConnectionSettings, RouterConfig, ServerSettings};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};

const CONNECT_PREFIX_TIMEOUT: Duration = Duration::from_secs(5);
const BROKER_START_TIMEOUT: Duration = Duration::from_secs(5);
const BROKER_START_RETRY_DELAY: Duration = Duration::from_millis(20);

pub fn run_child() -> i32 {
    match run_child_inner() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("broker error: {error}");
            1
        }
    }
}

fn run_child_inner() -> Result<(), String> {
    let config = BuiltInBrokerProcessConfig::read_from_child(io::stdin())
        .map_err(|error| format!("broker configuration could not be read: {error}"))?;
    let v4_port = reserve_loopback_port()?;
    let v5_port = reserve_loopback_port()?;
    let broker_config = broker_config(&config, v4_port, v5_port);

    let (engine_status_sender, engine_status_receiver) = mpsc::channel();
    thread::Builder::new()
        .name("correo-builtin-rumqttd".to_owned())
        .spawn(move || {
            let mut broker = Broker::new(broker_config);
            let result = broker
                .start()
                .map_err(|error| format!("broker engine stopped: {error}"));
            let _ = engine_status_sender.send(result);
        })
        .map_err(|error| format!("broker engine could not be spawned: {error}"))?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("broker proxy runtime could not be started: {error}"))?;
    runtime.block_on(async move {
        let listener = TokioTcpListener::bind(loopback(config.port))
            .await
            .map_err(|error| format!("broker could not bind 127.0.0.1:{}: {error}", config.port))?;
        wait_for_broker_listeners(v4_port, v5_port, &engine_status_receiver).await?;
        println!("{}", correo_core::builtin_broker_ready_message(config.port));
        flush_stdout();
        proxy_loop(listener, v4_port, v5_port).await
    })
}

fn broker_config(config: &BuiltInBrokerProcessConfig, v4_port: u16, v5_port: u16) -> Config {
    let auth = config.username.as_ref().map(|username| {
        HashMap::from([(
            username.clone(),
            config.password.clone().unwrap_or_default(),
        )])
    });
    let connections = ConnectionSettings {
        connection_timeout_ms: 10_000,
        max_payload_size: 10 * 1024 * 1024,
        max_inflight_count: 100,
        auth,
        external_auth: None,
        dynamic_filters: true,
    };

    Config {
        id: 0,
        router: RouterConfig {
            max_connections: 10_000,
            max_outgoing_packet_count: 10_000,
            max_segment_size: 10 * 1024 * 1024,
            max_segment_count: 10,
            custom_segment: None,
            initialized_filters: None,
            shared_subscriptions_strategy: Default::default(),
        },
        v4: Some(HashMap::from([(
            "mqtt3".to_owned(),
            server_settings("correo-mqtt3", v4_port, connections.clone()),
        )])),
        v5: Some(HashMap::from([(
            "mqtt5".to_owned(),
            server_settings("correo-mqtt5", v5_port, connections),
        )])),
        ws: None,
        cluster: None,
        console: None,
        bridge: None,
        prometheus: None,
        metrics: None,
    }
}

fn server_settings(name: &str, port: u16, connections: ConnectionSettings) -> ServerSettings {
    ServerSettings {
        name: name.to_owned(),
        listen: loopback(port),
        tls: None,
        next_connection_delay_ms: 1,
        connections,
    }
}

async fn proxy_loop(listener: TokioTcpListener, v4_port: u16, v5_port: u16) -> Result<(), String> {
    loop {
        let (client, peer) = listener
            .accept()
            .await
            .map_err(|error| format!("broker accept failed: {error}"))?;
        tokio::spawn(async move {
            if let Err(error) = proxy_client(client, v4_port, v5_port).await {
                eprintln!("broker client {peer} disconnected during setup: {error}");
            }
        });
    }
}

async fn wait_for_broker_listeners(
    v4_port: u16,
    v5_port: u16,
    engine_status: &mpsc::Receiver<Result<(), String>>,
) -> Result<(), String> {
    let deadline = Instant::now() + BROKER_START_TIMEOUT;
    loop {
        match engine_status.try_recv() {
            Ok(Ok(())) => return Err("broker engine stopped before reporting readiness".to_owned()),
            Ok(Err(error)) => return Err(error),
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err("broker engine stopped before reporting readiness".to_owned())
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }

        if listener_is_ready(v4_port).await && listener_is_ready(v5_port).await {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(
                "broker protocol listeners did not become ready within 5 seconds".to_owned(),
            );
        }
        tokio::time::sleep(BROKER_START_RETRY_DELAY).await;
    }
}

async fn listener_is_ready(port: u16) -> bool {
    tokio::time::timeout(BROKER_START_RETRY_DELAY, TcpStream::connect(loopback(port)))
        .await
        .is_ok_and(|result| result.is_ok())
}

async fn proxy_client(mut client: TcpStream, v4_port: u16, v5_port: u16) -> Result<(), String> {
    let (prefix, version) =
        tokio::time::timeout(CONNECT_PREFIX_TIMEOUT, read_connect_prefix(&mut client))
            .await
            .map_err(|_| "MQTT CONNECT header timed out".to_owned())??;
    let target_port = match version {
        MqttConnectVersion::V5 => v5_port,
        MqttConnectVersion::V3 => v4_port,
    };
    let mut broker = TcpStream::connect(loopback(target_port))
        .await
        .map_err(|error| format!("broker protocol listener unavailable: {error}"))?;
    broker
        .write_all(&prefix)
        .await
        .map_err(|error| format!("broker connect prefix could not be forwarded: {error}"))?;
    tokio::io::copy_bidirectional(&mut client, &mut broker)
        .await
        .map_err(|error| format!("broker proxy failed: {error}"))?;
    Ok(())
}

async fn read_connect_prefix(
    stream: &mut TcpStream,
) -> Result<(Vec<u8>, MqttConnectVersion), String> {
    let mut prefix = Vec::new();
    let packet_type = read_byte(stream, &mut prefix).await?;
    if packet_type != 0x10 {
        return Err("first MQTT packet is not CONNECT".to_owned());
    }

    for _ in 0..4 {
        let byte = read_byte(stream, &mut prefix).await?;
        if byte & 0x80 == 0 {
            break;
        }
    }

    let msb = read_byte(stream, &mut prefix).await? as usize;
    let lsb = read_byte(stream, &mut prefix).await? as usize;
    let protocol_name_len = (msb << 8) | lsb;
    if protocol_name_len > 16 {
        return Err("MQTT protocol name is invalid".to_owned());
    }

    let mut protocol_name = vec![0; protocol_name_len];
    stream
        .read_exact(&mut protocol_name)
        .await
        .map_err(|error| format!("MQTT protocol name could not be read: {error}"))?;
    prefix.extend_from_slice(&protocol_name);
    let level = read_byte(stream, &mut prefix).await?;

    match (protocol_name.as_slice(), level) {
        (b"MQTT", 5) => Ok((prefix, MqttConnectVersion::V5)),
        (b"MQTT", 4) | (b"MQIsdp", 3) => Ok((prefix, MqttConnectVersion::V3)),
        _ => Err(format!(
            "unsupported MQTT protocol level {level} for {}",
            String::from_utf8_lossy(&protocol_name)
        )),
    }
}

async fn read_byte(stream: &mut TcpStream, prefix: &mut Vec<u8>) -> Result<u8, String> {
    let mut byte = [0];
    stream
        .read_exact(&mut byte)
        .await
        .map_err(|error| format!("MQTT CONNECT header could not be read: {error}"))?;
    prefix.push(byte[0]);
    Ok(byte[0])
}

#[derive(Debug, Clone, Copy)]
enum MqttConnectVersion {
    V3,
    V5,
}

fn reserve_loopback_port() -> Result<u16, String> {
    TcpListener::bind(loopback(0))
        .map_err(|error| format!("broker internal port could not be reserved: {error}"))?
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|error| format!("broker internal port could not be inspected: {error}"))
}

fn loopback(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

fn flush_stdout() {
    let _ = io::stdout().flush();
}
