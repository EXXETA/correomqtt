use std::collections::HashMap;
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::thread;
use std::time::Duration;

use correo_core::BuiltInBrokerProcessConfig;
use rumqttd::{Broker, Config, ConnectionSettings, RouterConfig, ServerSettings};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};

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
    let config: BuiltInBrokerProcessConfig = serde_json::from_reader(io::stdin())
        .map_err(|error| format!("broker configuration could not be read: {error}"))?;
    let v4_port = reserve_loopback_port()?;
    let v5_port = reserve_loopback_port()?;
    let broker_config = broker_config(&config, v4_port, v5_port);

    thread::Builder::new()
        .name("correo-builtin-rumqttd".to_owned())
        .spawn(move || {
            let mut broker = Broker::new(broker_config);
            if let Err(error) = broker.start() {
                eprintln!("broker engine stopped: {error}");
            }
        })
        .map_err(|error| format!("broker engine could not be spawned: {error}"))?;

    thread::sleep(Duration::from_millis(200));
    println!(
        "broker engine ready; MQTT 3.1.1 and MQTT 5 are available on 127.0.0.1:{}",
        config.port
    );
    flush_stdout();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("broker proxy runtime could not be started: {error}"))?;
    runtime.block_on(proxy_loop(config.port, v4_port, v5_port))
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

async fn proxy_loop(public_port: u16, v4_port: u16, v5_port: u16) -> Result<(), String> {
    let listener = TokioTcpListener::bind(loopback(public_port))
        .await
        .map_err(|error| format!("broker could not bind 127.0.0.1:{public_port}: {error}"))?;
    println!("broker proxy listening on 127.0.0.1:{public_port}");
    flush_stdout();

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

async fn proxy_client(mut client: TcpStream, v4_port: u16, v5_port: u16) -> Result<(), String> {
    let (prefix, version) = read_connect_prefix(&mut client).await?;
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
