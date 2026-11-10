use std::time::Duration;

use correo_mqtt::{
    ConnectionId, LastWill, Mqtt311Session, Mqtt5Session, MqttAuth, MqttConnectionOptions,
    MqttEndpoint, MqttProtocolVersion, MqttSession, PublishRequest, Qos, SecretString,
    Subscription, TopicName, UnsubscribeRequest,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn mqtt3_session_writes_expected_packets_to_broker() {
    run_packet_mapping_test(MqttProtocolVersion::Mqtt3_1_1).await;
}

#[tokio::test]
async fn mqtt5_session_writes_expected_packets_to_broker() {
    run_packet_mapping_test(MqttProtocolVersion::Mqtt5).await;
}

async fn run_packet_mapping_test(protocol: MqttProtocolVersion) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let broker = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        handle_connect(&mut socket, protocol).await;
        handle_publish(&mut socket, protocol).await;
        handle_subscribe(&mut socket, protocol).await;
        handle_unsubscribe(&mut socket, protocol).await;
        assert_eq!(read_packet(&mut socket).await[0] >> 4, 14);
    });

    match protocol {
        MqttProtocolVersion::Mqtt3_1_1 => {
            let mut session = Mqtt311Session::new();
            exercise_session(&mut session, protocol, port).await;
        }
        MqttProtocolVersion::Mqtt5 => {
            let mut session = Mqtt5Session::new();
            exercise_session(&mut session, protocol, port).await;
        }
    }

    tokio::time::timeout(Duration::from_secs(5), broker)
        .await
        .unwrap()
        .unwrap();
}

async fn exercise_session(session: &mut dyn MqttSession, protocol: MqttProtocolVersion, port: u16) {
    session.connect(options(protocol, port)).await.unwrap();
    session
        .publish(
            PublishRequest::new(
                "devices/alpha/state",
                b"online".to_vec(),
                Qos::AtLeastOnce,
                true,
            )
            .unwrap(),
        )
        .await
        .unwrap();
    session
        .subscribe(Subscription::new("devices/+/state", Qos::ExactlyOnce).unwrap())
        .await
        .unwrap();
    session
        .unsubscribe(UnsubscribeRequest::new("devices/+/state").unwrap())
        .await
        .unwrap();
    session.disconnect().await.unwrap();
}

fn options(protocol: MqttProtocolVersion, port: u16) -> MqttConnectionOptions {
    let mut options = MqttConnectionOptions::new(
        ConnectionId::new(),
        "fake broker",
        MqttEndpoint::new("127.0.0.1", port).unwrap(),
    );
    options.protocol_version = protocol;
    options.client_id = Some("correo-test-client".to_owned());
    options.keep_alive = Duration::from_secs(5);
    options.auth = MqttAuth::UsernamePassword {
        username: Some("synthetic-user".to_owned()),
        password: SecretString::new("synthetic-password"),
    };
    options.last_will = Some(LastWill {
        topic: TopicName::new("devices/alpha/will").unwrap(),
        payload: b"offline".to_vec(),
        qos: Qos::AtLeastOnce,
        retain: true,
    });
    options
}

async fn handle_connect(socket: &mut TcpStream, protocol: MqttProtocolVersion) {
    let packet = read_packet(socket).await;
    assert_eq!(packet[0] >> 4, 1);
    assert_connect(&packet, protocol);
    match protocol {
        MqttProtocolVersion::Mqtt3_1_1 => socket.write_all(&[0x20, 0x02, 0x00, 0x00]).await,
        MqttProtocolVersion::Mqtt5 => socket.write_all(&[0x20, 0x03, 0x00, 0x00, 0x00]).await,
    }
    .unwrap();
}

async fn handle_publish(socket: &mut TcpStream, protocol: MqttProtocolVersion) {
    let packet = read_packet(socket).await;
    assert_publish(&packet, protocol);
    let pkid = packet_id(&packet, protocol);
    match protocol {
        MqttProtocolVersion::Mqtt3_1_1 => socket.write_all(&[0x40, 0x02, pkid[0], pkid[1]]).await,
        MqttProtocolVersion::Mqtt5 => {
            socket
                .write_all(&[0x40, 0x04, pkid[0], pkid[1], 0x00, 0x00])
                .await
        }
    }
    .unwrap();
}

async fn handle_subscribe(socket: &mut TcpStream, protocol: MqttProtocolVersion) {
    let packet = read_packet(socket).await;
    assert_subscribe(&packet, protocol);
    let pkid = [body(&packet)[0], body(&packet)[1]];
    match protocol {
        MqttProtocolVersion::Mqtt3_1_1 => {
            socket
                .write_all(&[0x90, 0x03, pkid[0], pkid[1], 0x02])
                .await
        }
        MqttProtocolVersion::Mqtt5 => {
            socket
                .write_all(&[0x90, 0x04, pkid[0], pkid[1], 0x00, 0x02])
                .await
        }
    }
    .unwrap();
}

async fn handle_unsubscribe(socket: &mut TcpStream, protocol: MqttProtocolVersion) {
    let packet = read_packet(socket).await;
    assert_unsubscribe(&packet, protocol);
    let pkid = [body(&packet)[0], body(&packet)[1]];
    match protocol {
        MqttProtocolVersion::Mqtt3_1_1 => socket.write_all(&[0xB0, 0x02, pkid[0], pkid[1]]).await,
        MqttProtocolVersion::Mqtt5 => {
            socket
                .write_all(&[0xB0, 0x04, pkid[0], pkid[1], 0x00, 0x00])
                .await
        }
    }
    .unwrap();
}

async fn read_packet(socket: &mut TcpStream) -> Vec<u8> {
    let mut packet = vec![socket.read_u8().await.unwrap()];
    let mut multiplier = 1usize;
    let mut remaining = 0usize;
    loop {
        let byte = socket.read_u8().await.unwrap();
        packet.push(byte);
        remaining += usize::from(byte & 0x7F) * multiplier;
        if byte & 0x80 == 0 {
            break;
        }
        multiplier *= 128;
    }
    let start = packet.len();
    packet.resize(start + remaining, 0);
    socket.read_exact(&mut packet[start..]).await.unwrap();
    packet
}

fn body(packet: &[u8]) -> &[u8] {
    let mut cursor = 1;
    while packet[cursor] & 0x80 != 0 {
        cursor += 1;
    }
    &packet[cursor + 1..]
}

fn assert_connect(packet: &[u8], protocol: MqttProtocolVersion) {
    let mut cursor = Cursor::new(body(packet));
    assert_eq!(cursor.string(), "MQTT");
    assert_eq!(cursor.u8(), protocol.wire_level());
    let flags = cursor.u8();
    assert!(flags & 0x02 != 0);
    assert!(flags & 0x04 != 0);
    assert!(flags & 0x20 != 0);
    assert!(flags & 0x40 != 0);
    assert!(flags & 0x80 != 0);
    assert_eq!(cursor.u16(), 5);
    if protocol == MqttProtocolVersion::Mqtt5 {
        const MAXIMUM_PACKET_SIZE_PROPERTY: u8 = 0x27;
        assert_eq!(cursor.varint(), 5);
        assert_eq!(cursor.u8(), MAXIMUM_PACKET_SIZE_PROPERTY);
        let maximum_packet_size = (u32::from(cursor.u16()) << 16) | u32::from(cursor.u16());
        assert!(maximum_packet_size > 0);
    }
    assert_eq!(cursor.string(), "correo-test-client");
    if protocol == MqttProtocolVersion::Mqtt5 {
        assert_eq!(cursor.varint(), 0);
    }
    assert_eq!(cursor.string(), "devices/alpha/will");
    assert_eq!(cursor.bytes(), b"offline");
    assert_eq!(cursor.string(), "synthetic-user");
    assert_eq!(cursor.bytes(), b"synthetic-password");
}

fn assert_publish(packet: &[u8], protocol: MqttProtocolVersion) {
    assert_eq!(packet[0] >> 4, 3);
    assert!(packet[0] & 0x01 != 0);
    assert_eq!((packet[0] & 0x06) >> 1, 1);
    let mut cursor = Cursor::new(body(packet));
    assert_eq!(cursor.string(), "devices/alpha/state");
    assert_ne!(cursor.u16(), 0);
    if protocol == MqttProtocolVersion::Mqtt5 {
        assert_eq!(cursor.varint(), 0);
    }
    assert_eq!(cursor.remaining(), b"online");
}

fn assert_subscribe(packet: &[u8], protocol: MqttProtocolVersion) {
    assert_eq!(packet[0], 0x82);
    let mut cursor = Cursor::new(body(packet));
    assert_ne!(cursor.u16(), 0);
    if protocol == MqttProtocolVersion::Mqtt5 {
        assert_eq!(cursor.varint(), 0);
    }
    assert_eq!(cursor.string(), "devices/+/state");
    assert_eq!(cursor.u8(), 2);
}

fn assert_unsubscribe(packet: &[u8], protocol: MqttProtocolVersion) {
    assert_eq!(packet[0], 0xA2);
    let mut cursor = Cursor::new(body(packet));
    assert_ne!(cursor.u16(), 0);
    if protocol == MqttProtocolVersion::Mqtt5 {
        assert_eq!(cursor.varint(), 0);
    }
    assert_eq!(cursor.string(), "devices/+/state");
}

fn packet_id(packet: &[u8], protocol: MqttProtocolVersion) -> [u8; 2] {
    let mut cursor = Cursor::new(body(packet));
    let _ = cursor.string();
    let pkid = cursor.u16();
    if protocol == MqttProtocolVersion::Mqtt5 {
        let _ = cursor.varint();
    }
    pkid.to_be_bytes()
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn u8(&mut self) -> u8 {
        let value = self.bytes[self.position];
        self.position += 1;
        value
    }

    fn u16(&mut self) -> u16 {
        let value = u16::from_be_bytes([self.bytes[self.position], self.bytes[self.position + 1]]);
        self.position += 2;
        value
    }

    fn string(&mut self) -> String {
        String::from_utf8(self.bytes().to_vec()).unwrap()
    }

    fn bytes(&mut self) -> &'a [u8] {
        let len = usize::from(self.u16());
        let start = self.position;
        self.position += len;
        &self.bytes[start..start + len]
    }

    fn varint(&mut self) -> usize {
        let mut multiplier = 1usize;
        let mut value = 0usize;
        loop {
            let byte = self.u8();
            value += usize::from(byte & 0x7F) * multiplier;
            if byte & 0x80 == 0 {
                return value;
            }
            multiplier *= 128;
        }
    }

    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.position..]
    }
}
