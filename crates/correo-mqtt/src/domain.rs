use std::convert::TryFrom;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    error::{MqttError, MqttErrorReport, MqttResult},
    id::ConnectionId,
    secret::{SecretBytes, SecretString},
};

pub type MqttProtocol = MqttProtocolVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MqttProtocolVersion {
    Mqtt3_1_1,
    Mqtt5,
}

impl MqttProtocolVersion {
    pub fn wire_level(self) -> u8 {
        match self {
            Self::Mqtt3_1_1 => 4,
            Self::Mqtt5 => 5,
        }
    }
}

impl std::fmt::Display for MqttProtocolVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Mqtt3_1_1 => "MQTT 3.1.1",
            Self::Mqtt5 => "MQTT 5",
        };
        formatter.write_str(label)
    }
}

impl TryFrom<&str> for MqttProtocolVersion {
    type Error = MqttError;

    fn try_from(value: &str) -> MqttResult<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "3" | "3.1.1" | "mqtt3" | "mqtt_3_1_1" | "mqtt 3.1.1" => Ok(Self::Mqtt3_1_1),
            "5" | "5.0" | "mqtt5" | "mqtt_5" | "mqtt 5" | "mqtt v5" => Ok(Self::Mqtt5),
            other => Err(MqttError::protocol(format!(
                "unsupported MQTT protocol version: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Qos {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl From<Qos> for u8 {
    fn from(qos: Qos) -> Self {
        match qos {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2,
        }
    }
}

impl TryFrom<u8> for Qos {
    type Error = MqttError;

    fn try_from(value: u8) -> MqttResult<Self> {
        match value {
            0 => Ok(Self::AtMostOnce),
            1 => Ok(Self::AtLeastOnce),
            2 => Ok(Self::ExactlyOnce),
            _ => Err(MqttError::protocol(format!(
                "invalid MQTT QoS value: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicName(String);

impl TopicName {
    pub fn new(value: impl Into<String>) -> MqttResult<Self> {
        let value = value.into();
        validate_topic_name(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for TopicName {
    type Error = MqttError;

    fn try_from(value: &str) -> MqttResult<Self> {
        Self::new(value)
    }
}

impl std::fmt::Display for TopicName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicFilter(String);

impl TopicFilter {
    pub fn new(value: impl Into<String>) -> MqttResult<Self> {
        let value = value.into();
        validate_topic_filter(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for TopicFilter {
    type Error = MqttError;

    fn try_from(value: &str) -> MqttResult<Self> {
        Self::new(value)
    }
}

impl std::fmt::Display for TopicFilter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttEndpoint {
    pub host: String,
    pub port: u16,
}

impl MqttEndpoint {
    pub fn new(host: impl Into<String>, port: u16) -> MqttResult<Self> {
        let host = host.into();
        if host.trim().is_empty() {
            return Err(MqttError::invalid_options("MQTT host is required"));
        }
        if port == 0 {
            return Err(MqttError::invalid_options(
                "MQTT port must be greater than zero",
            ));
        }
        Ok(Self { host, port })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttConnectionOptions {
    pub connection_id: ConnectionId,
    pub name: String,
    pub endpoint: MqttEndpoint,
    pub client_id: Option<String>,
    pub protocol_version: MqttProtocolVersion,
    pub clean_start: bool,
    pub keep_alive: Duration,
    pub auth: MqttAuth,
    pub tls: TlsConfig,
    pub ssh_tunnel: Option<SshTunnelOptions>,
    pub last_will: Option<LastWill>,
}

impl MqttConnectionOptions {
    pub fn new(
        connection_id: ConnectionId,
        name: impl Into<String>,
        endpoint: MqttEndpoint,
    ) -> Self {
        Self {
            connection_id,
            name: name.into(),
            endpoint,
            client_id: None,
            protocol_version: MqttProtocolVersion::Mqtt3_1_1,
            clean_start: true,
            keep_alive: Duration::from_secs(30),
            auth: MqttAuth::Anonymous,
            tls: TlsConfig::Disabled,
            ssh_tunnel: None,
            last_will: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MqttAuth {
    Anonymous,
    UsernamePassword {
        username: Option<String>,
        password: SecretString,
    },
    Token {
        token: SecretString,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsConfig {
    Disabled,
    Enabled(TlsOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsOptions {
    pub host_verification: TlsHostVerification,
    pub trust_roots: TlsTrustRoots,
    pub client_identity: Option<TlsClientIdentity>,
}

impl Default for TlsOptions {
    fn default() -> Self {
        Self {
            host_verification: TlsHostVerification::Enabled,
            trust_roots: TlsTrustRoots::Native,
            client_identity: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TlsHostVerification {
    Enabled,
    DisabledInsecure,
}

impl TlsHostVerification {
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsTrustRoots {
    Native,
    PemBundle {
        path: Option<String>,
        pem: Option<SecretBytes>,
    },
    Pkcs12 {
        path: Option<String>,
        der: SecretBytes,
        password: Option<SecretString>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsClientIdentity {
    Pem {
        certificate_pem: SecretBytes,
        private_key_pem: SecretBytes,
    },
    Pkcs12 {
        der: SecretBytes,
        password: Option<SecretString>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTunnelOptions {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
    pub host_key_policy: SshHostKeyPolicy,
    pub local_bind_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SshAuth {
    Password(SecretString),
    PrivateKey {
        path: Option<String>,
        private_key: Option<SecretBytes>,
        passphrase: Option<SecretString>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SshHostKeyPolicy {
    AcceptAnyInsecure,
    TrustOnFirstUse {
        known_hosts_path: std::path::PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastWill {
    pub topic: TopicName,
    pub payload: Vec<u8>,
    pub qos: Qos,
    pub retain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishRequest {
    pub topic: TopicName,
    pub payload: Vec<u8>,
    pub qos: Qos,
    pub retain: bool,
}

impl PublishRequest {
    pub fn new(
        topic: impl TryInto<TopicName, Error = MqttError>,
        payload: impl Into<Vec<u8>>,
        qos: Qos,
        retain: bool,
    ) -> MqttResult<Self> {
        Ok(Self {
            topic: topic.try_into()?,
            payload: payload.into(),
            qos,
            retain,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscription {
    pub topic_filter: TopicFilter,
    pub qos: Qos,
}

impl Subscription {
    pub fn new(
        topic_filter: impl TryInto<TopicFilter, Error = MqttError>,
        qos: Qos,
    ) -> MqttResult<Self> {
        Ok(Self {
            topic_filter: topic_filter.try_into()?,
            qos,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsubscribeRequest {
    pub topic_filter: TopicFilter,
}

impl UnsubscribeRequest {
    pub fn new(topic_filter: impl TryInto<TopicFilter, Error = MqttError>) -> MqttResult<Self> {
        Ok(Self {
            topic_filter: topic_filter.try_into()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub connection_id: ConnectionId,
    pub topic: TopicName,
    pub payload: Vec<u8>,
    pub qos: Qos,
    pub retain: bool,
    pub duplicate: bool,
    pub packet_id: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Reconnecting { attempt: u32 },
    Faulted { error: MqttErrorReport },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MqttSessionEvent {
    StateChanged(SessionState),
    Incoming(IncomingMessage),
    Published {
        topic: TopicName,
        payload: Vec<u8>,
        qos: Qos,
        retain: bool,
    },
    Subscribed(Subscription),
    Unsubscribed(UnsubscribeRequest),
    Error(MqttErrorReport),
}

fn validate_topic_name(value: &str) -> MqttResult<()> {
    if value.is_empty() {
        return Err(MqttError::invalid_options("MQTT topic name is required"));
    }
    if value.contains('+') || value.contains('#') {
        return Err(MqttError::invalid_options(
            "MQTT publish topic names cannot contain wildcards",
        ));
    }
    Ok(())
}

fn validate_topic_filter(value: &str) -> MqttResult<()> {
    if value.is_empty() {
        return Err(MqttError::invalid_options("MQTT topic filter is required"));
    }

    for level in value.split('/') {
        if level.contains('+') && level != "+" {
            return Err(MqttError::invalid_options(
                "MQTT + wildcard must occupy a complete topic level",
            ));
        }
    }

    if let Some(index) = value.find('#') {
        let hash_is_last = index + 1 == value.len();
        let hash_starts_level = index == 0 || value.as_bytes()[index - 1] == b'/';
        if !hash_is_last || !hash_starts_level {
            return Err(MqttError::invalid_options(
                "MQTT # wildcard must be the final topic level",
            ));
        }
    }

    Ok(())
}
