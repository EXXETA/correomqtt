use std::{env, fs, time::Duration};

use correo_mqtt::{
    ConnectionId, IncomingMessage, Mqtt311Session, Mqtt5Session, MqttConnectionOptions,
    MqttEndpoint, MqttError, MqttProtocolVersion, MqttSession, MqttSessionEvent, PublishRequest,
    Qos, SecretBytes, SessionState, Subscription, TlsConfig, TlsHostVerification, TlsOptions,
    TlsTrustRoots,
};
use futures::{stream::BoxStream, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const TEST_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test]
#[ignore = "requires a local broker; see crates/correo-mqtt/docs/local-broker-integration.md"]
async fn mqtt3_local_broker_matrix() {
    let Some(broker) = BrokerConfig::plain() else {
        return;
    };
    run_broker_matrix(MqttProtocolVersion::Mqtt3_1_1, &broker).await;
}

#[tokio::test]
#[ignore = "requires a local broker; see crates/correo-mqtt/docs/local-broker-integration.md"]
async fn mqtt5_local_broker_matrix() {
    let Some(broker) = BrokerConfig::plain() else {
        return;
    };
    run_broker_matrix(MqttProtocolVersion::Mqtt5, &broker).await;
}

#[tokio::test]
#[ignore = "requires a TLS broker; see crates/correo-mqtt/docs/local-broker-integration.md"]
async fn mqtt3_tls_connect_disconnect() {
    let Some(broker) = BrokerConfig::tls() else {
        return;
    };
    assert_connect_disconnect(MqttProtocolVersion::Mqtt3_1_1, &broker, "tls").await;
}

#[tokio::test]
#[ignore = "requires a TLS broker; see crates/correo-mqtt/docs/local-broker-integration.md"]
async fn mqtt5_tls_connect_disconnect() {
    let Some(broker) = BrokerConfig::tls() else {
        return;
    };
    assert_connect_disconnect(MqttProtocolVersion::Mqtt5, &broker, "tls").await;
}

#[tokio::test]
#[ignore = "requires a TLS broker; see crates/correo-mqtt/docs/local-broker-integration.md"]
async fn tls_rejects_untrusted_certificates_and_hostname_mismatches() {
    let Some(broker) = BrokerConfig::tls() else {
        return;
    };
    let Some(untrusted_broker) = broker.with_ca_from_env("CORREO_MQTT_TLS_UNTRUSTED_CA_PEM") else {
        return;
    };
    let hostname_mismatch_broker = broker.with_host("127.0.0.2");

    for protocol in [MqttProtocolVersion::Mqtt3_1_1, MqttProtocolVersion::Mqtt5] {
        assert_connect_fails(protocol, &untrusted_broker, "untrusted CA").await;
        assert_connect_fails(protocol, &hostname_mismatch_broker, "hostname mismatch").await;
    }
}

#[tokio::test]
#[ignore = "uses a local disconnect probe for reconnect reporting"]
async fn reconnect_reporting_uses_local_disconnect_probe() {
    assert_reconnect_reporting(MqttProtocolVersion::Mqtt3_1_1).await;
    assert_reconnect_reporting(MqttProtocolVersion::Mqtt5).await;
}

async fn run_broker_matrix(protocol: MqttProtocolVersion, broker: &BrokerConfig) {
    let prefix = format!(
        "correo/integration/{}/{}",
        protocol_label(protocol),
        ConnectionId::new()
    );

    assert_connect_disconnect(protocol, broker, "plain").await;
    for qos in [Qos::AtMostOnce, Qos::AtLeastOnce, Qos::ExactlyOnce] {
        assert_publish_subscribe(protocol, broker, &prefix, qos).await;
    }
    assert_retained_message(protocol, broker, &prefix).await;
}

async fn assert_connect_disconnect(
    protocol: MqttProtocolVersion,
    broker: &BrokerConfig,
    purpose: &str,
) {
    let mut session = new_session(protocol);
    session
        .connect(options(protocol, broker, purpose))
        .await
        .unwrap_or_else(|error| panic!("connect failed for {protocol}: {error}"));
    assert_eq!(session.current_state(), SessionState::Connected);
    session
        .disconnect()
        .await
        .unwrap_or_else(|error| panic!("disconnect failed for {protocol}: {error}"));
    assert_eq!(session.current_state(), SessionState::Disconnected);
}

async fn assert_connect_fails(protocol: MqttProtocolVersion, broker: &BrokerConfig, purpose: &str) {
    tokio::time::timeout(
        TEST_TIMEOUT,
        TcpStream::connect((broker.host.as_str(), broker.port)),
    )
    .await
    .unwrap_or_else(|_| panic!("TLS listener was unreachable for {purpose}"))
    .unwrap_or_else(|error| panic!("TLS listener was unreachable for {purpose}: {error}"));

    let mut session = new_session(protocol);
    let result = tokio::time::timeout(
        TEST_TIMEOUT,
        session.connect(options(protocol, broker, purpose)),
    )
    .await
    .unwrap_or_else(|_| panic!("TLS {purpose} connection timed out for {protocol}"));
    assert!(
        result.is_err(),
        "TLS {purpose} connection unexpectedly succeeded for {protocol}"
    );
}

async fn assert_publish_subscribe(
    protocol: MqttProtocolVersion,
    broker: &BrokerConfig,
    prefix: &str,
    qos: Qos,
) {
    let topic = format!("{prefix}/qos/{}", qos_label(qos));
    let payload = format!("payload-{protocol}-{}", qos_label(qos)).into_bytes();
    let mut session = new_session(protocol);

    session
        .connect(options(protocol, broker, "qos"))
        .await
        .unwrap_or_else(|error| panic!("connect failed for {protocol}: {error}"));
    let mut incoming = session.incoming();
    session
        .subscribe(Subscription::new(topic.as_str(), qos).expect("valid subscription"))
        .await
        .unwrap_or_else(|error| panic!("subscribe failed for {protocol} {qos:?}: {error}"));
    session
        .publish(
            PublishRequest::new(topic.as_str(), payload.clone(), qos, false)
                .expect("valid publish"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish failed for {protocol} {qos:?}: {error}"));

    let message = next_matching(&mut incoming, &topic, &payload).await;
    assert_eq!(message.qos, qos);
    assert!(!message.retain);
    session.disconnect().await.expect("disconnect");
}

async fn assert_retained_message(
    protocol: MqttProtocolVersion,
    broker: &BrokerConfig,
    prefix: &str,
) {
    let topic = format!("{prefix}/retained");
    let payload = format!("retained-{protocol}-{}", ConnectionId::new()).into_bytes();
    let mut publisher = new_session(protocol);

    publisher
        .connect(options(protocol, broker, "retained-publisher"))
        .await
        .unwrap_or_else(|error| panic!("connect failed for {protocol}: {error}"));
    let mut publisher_incoming = publisher.incoming();
    publisher
        .subscribe(Subscription::new(topic.as_str(), Qos::AtLeastOnce).expect("valid subscribe"))
        .await
        .expect("subscribe retained topic");
    publisher
        .publish(
            PublishRequest::new(topic.as_str(), payload.clone(), Qos::AtLeastOnce, true)
                .expect("valid publish"),
        )
        .await
        .expect("publish retained message");
    let live_message = next_matching(&mut publisher_incoming, &topic, &payload).await;
    assert!(!live_message.retain);
    publisher.disconnect().await.expect("publisher disconnect");

    let mut subscriber = new_session(protocol);
    subscriber
        .connect(options(protocol, broker, "retained-subscriber"))
        .await
        .unwrap_or_else(|error| panic!("connect failed for {protocol}: {error}"));
    let mut subscriber_incoming = subscriber.incoming();
    subscriber
        .subscribe(Subscription::new(topic.as_str(), Qos::AtLeastOnce).expect("valid subscribe"))
        .await
        .expect("subscribe retained topic");
    let retained_message = next_matching(&mut subscriber_incoming, &topic, &payload).await;
    assert!(retained_message.retain);
    subscriber
        .disconnect()
        .await
        .expect("subscriber disconnect");

    clear_retained(protocol, broker, &topic).await;
}

async fn clear_retained(protocol: MqttProtocolVersion, broker: &BrokerConfig, topic: &str) {
    let mut session = new_session(protocol);
    session
        .connect(options(protocol, broker, "retained-cleanup"))
        .await
        .expect("connect retained cleanup");
    session
        .publish(PublishRequest::new(topic, Vec::new(), Qos::AtMostOnce, true).expect("clear"))
        .await
        .expect("clear retained message");
    tokio::time::sleep(Duration::from_millis(100)).await;
    session.disconnect().await.expect("cleanup disconnect");
}

async fn assert_reconnect_reporting(protocol: MqttProtocolVersion) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("bind probe broker");
    let port = listener.local_addr().expect("local addr").port();
    let broker_task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept probe client");
        let packet = read_packet(&mut socket).await;
        assert_eq!(packet[0] >> 4, 1);
        write_connack(&mut socket, protocol).await;
    });

    let broker = BrokerConfig {
        host: "127.0.0.1".to_owned(),
        port,
        tls: TlsConfig::Disabled,
    };
    let mut session = new_session(protocol);
    let mut events = session.events();
    session
        .connect(options(protocol, &broker, "reconnect"))
        .await
        .unwrap_or_else(|error| panic!("probe connect failed for {protocol}: {error}"));
    let attempt = next_reconnect_attempt(&mut events).await;
    assert_eq!(attempt, 1);
    let _ = session.disconnect().await;
    broker_task.await.expect("probe broker task");
}

async fn next_matching(
    incoming: &mut BoxStream<'static, Result<IncomingMessage, MqttError>>,
    topic: &str,
    payload: &[u8],
) -> IncomingMessage {
    tokio::time::timeout(TEST_TIMEOUT, async {
        while let Some(item) = incoming.next().await {
            let message = item.expect("incoming message error");
            if message.topic.as_str() == topic && message.payload == payload {
                return message;
            }
        }
        panic!("incoming stream ended before receiving {topic}");
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for {topic}"))
}

async fn next_reconnect_attempt(events: &mut BoxStream<'static, MqttSessionEvent>) -> u32 {
    tokio::time::timeout(TEST_TIMEOUT, async {
        while let Some(event) = events.next().await {
            if let MqttSessionEvent::StateChanged(SessionState::Reconnecting { attempt }) = event {
                return attempt;
            }
        }
        panic!("event stream ended before reconnect reporting");
    })
    .await
    .expect("timed out waiting for reconnect reporting")
}

fn new_session(protocol: MqttProtocolVersion) -> Box<dyn MqttSession> {
    match protocol {
        MqttProtocolVersion::Mqtt3_1_1 => Box::new(Mqtt311Session::new()),
        MqttProtocolVersion::Mqtt5 => Box::new(Mqtt5Session::new()),
    }
}

fn options(
    protocol: MqttProtocolVersion,
    broker: &BrokerConfig,
    purpose: &str,
) -> MqttConnectionOptions {
    let mut options = MqttConnectionOptions::new(
        ConnectionId::new(),
        format!("local broker {purpose}"),
        MqttEndpoint::new(broker.host.clone(), broker.port).expect("valid broker endpoint"),
    );
    options.protocol_version = protocol;
    options.client_id = Some(format!(
        "correo-{purpose}-{}-{}",
        protocol_label(protocol),
        ConnectionId::new()
    ));
    options.keep_alive = Duration::from_secs(5);
    options.tls = broker.tls.clone();
    options
}

async fn read_packet(socket: &mut TcpStream) -> Vec<u8> {
    let mut packet = vec![socket.read_u8().await.expect("fixed header")];
    let mut multiplier = 1usize;
    let mut remaining = 0usize;
    loop {
        let byte = socket.read_u8().await.expect("remaining length");
        packet.push(byte);
        remaining += usize::from(byte & 0x7F) * multiplier;
        if byte & 0x80 == 0 {
            break;
        }
        multiplier *= 128;
    }
    let start = packet.len();
    packet.resize(start + remaining, 0);
    socket
        .read_exact(&mut packet[start..])
        .await
        .expect("packet body");
    packet
}

async fn write_connack(socket: &mut TcpStream, protocol: MqttProtocolVersion) {
    let bytes: &[u8] = match protocol {
        MqttProtocolVersion::Mqtt3_1_1 => &[0x20, 0x02, 0x00, 0x00],
        MqttProtocolVersion::Mqtt5 => &[0x20, 0x03, 0x00, 0x00, 0x00],
    };
    socket.write_all(bytes).await.expect("write connack");
}

fn protocol_label(protocol: MqttProtocolVersion) -> &'static str {
    match protocol {
        MqttProtocolVersion::Mqtt3_1_1 => "mqtt3",
        MqttProtocolVersion::Mqtt5 => "mqtt5",
    }
}

fn qos_label(qos: Qos) -> &'static str {
    match qos {
        Qos::AtMostOnce => "qos0",
        Qos::AtLeastOnce => "qos1",
        Qos::ExactlyOnce => "qos2",
    }
}

#[derive(Clone)]
struct BrokerConfig {
    host: String,
    port: u16,
    tls: TlsConfig,
}

impl BrokerConfig {
    fn plain() -> Option<Self> {
        if !env_flag("CORREO_MQTT_INTEGRATION_BROKER") {
            skip("set CORREO_MQTT_INTEGRATION_BROKER=1 to run broker tests");
            return None;
        }
        Some(Self {
            host: env::var("CORREO_MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_owned()),
            port: env_port("CORREO_MQTT_BROKER_PORT", 1883),
            tls: TlsConfig::Disabled,
        })
    }

    fn tls() -> Option<Self> {
        if !env_flag("CORREO_MQTT_INTEGRATION_BROKER") {
            skip("set CORREO_MQTT_INTEGRATION_BROKER=1 to run TLS broker tests");
            return None;
        }
        let ca_path = match env::var("CORREO_MQTT_TLS_CA_PEM") {
            Ok(path) if !path.trim().is_empty() => path,
            _ => {
                skip("set CORREO_MQTT_TLS_CA_PEM to the synthetic CA certificate");
                return None;
            }
        };
        let ca_pem = fs::read(&ca_path).unwrap_or_else(|error| {
            panic!("failed to read CORREO_MQTT_TLS_CA_PEM={ca_path}: {error}")
        });
        Some(Self {
            host: env::var("CORREO_MQTT_TLS_HOST").unwrap_or_else(|_| "localhost".to_owned()),
            port: env_port("CORREO_MQTT_TLS_BROKER_PORT", 8883),
            tls: TlsConfig::Enabled(TlsOptions {
                host_verification: TlsHostVerification::Enabled,
                trust_roots: TlsTrustRoots::PemBundle {
                    path: Some(ca_path),
                    pem: Some(SecretBytes::new(ca_pem)),
                },
                client_identity: None,
            }),
        })
    }

    fn with_ca_from_env(&self, name: &str) -> Option<Self> {
        let ca_path = match env::var(name) {
            Ok(path) if !path.trim().is_empty() => path,
            _ => {
                skip(&format!("set {name} to a synthetic CA certificate"));
                return None;
            }
        };
        let ca_pem = fs::read(&ca_path)
            .unwrap_or_else(|error| panic!("failed to read {name}={ca_path}: {error}"));
        Some(Self {
            host: self.host.clone(),
            port: self.port,
            tls: TlsConfig::Enabled(TlsOptions {
                host_verification: TlsHostVerification::Enabled,
                trust_roots: TlsTrustRoots::PemBundle {
                    path: Some(ca_path),
                    pem: Some(SecretBytes::new(ca_pem)),
                },
                client_identity: None,
            }),
        })
    }

    fn with_host(&self, host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: self.port,
            tls: self.tls.clone(),
        }
    }
}

fn env_flag(name: &str) -> bool {
    matches!(
        env::var(name).ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON")
    )
}

fn env_port(name: &str, default: u16) -> u16 {
    match env::var(name) {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|_| panic!("{name} must be a valid TCP port")),
        Err(_) => default,
    }
}

fn skip(reason: &str) {
    eprintln!("skipped correo-mqtt local broker integration: {reason}");
}
