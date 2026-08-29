use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespacedName(String);

impl NamespacedName {
    pub fn new(value: impl Into<String>) -> Result<Self, TransportValueError> {
        let value = value.into();
        let Some((namespace, name)) = value.split_once('.') else {
            return Err(TransportValueError::MissingNamespace { value });
        };
        if namespace.is_empty() || name.is_empty() {
            return Err(TransportValueError::MissingNamespace { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransportConnectionId(String);

impl TransportConnectionId {
    pub fn new(value: impl Into<String>) -> Result<Self, TransportValueError> {
        let value = value.into();
        if value.is_empty() {
            return Err(TransportValueError::EmptyConnectionId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProtocolMetadata(BTreeMap<NamespacedName, Value>);

impl ProtocolMetadata {
    pub fn insert(&mut self, key: NamespacedName, value: Value) -> Option<Value> {
        self.0.insert(key, value)
    }

    pub fn get(&self, key: &NamespacedName) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn get_value(&self, key: &str) -> Option<&Value> {
        NamespacedName::new(key).ok().and_then(|key| self.get(&key))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NamespacedName, &Value)> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransportCapability(NamespacedName);

impl TransportCapability {
    pub fn new(value: impl Into<String>) -> Result<Self, TransportValueError> {
        NamespacedName::new(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransportCapabilities(BTreeSet<TransportCapability>);

impl TransportCapabilities {
    pub fn contains(&self, capability: &TransportCapability) -> bool {
        self.0.contains(capability)
    }

    pub fn iter(&self) -> impl Iterator<Item = &TransportCapability> {
        self.0.iter()
    }

    pub fn insert(&mut self, capability: TransportCapability) -> bool {
        self.0.insert(capability)
    }
}

impl<const N: usize> From<[TransportCapability; N]> for TransportCapabilities {
    fn from(capabilities: [TransportCapability; N]) -> Self {
        Self(BTreeSet::from(capabilities))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryGuarantee {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliverySemantics {
    pub guarantee: DeliveryGuarantee,
}

impl DeliverySemantics {
    pub fn new(guarantee: DeliveryGuarantee) -> Self {
        Self { guarantee }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageEnvelope {
    pub address: String,
    pub body: Vec<u8>,
    pub delivery: DeliverySemantics,
    pub metadata: ProtocolMetadata,
}

impl MessageEnvelope {
    pub fn new(
        address: impl Into<String>,
        body: impl Into<Vec<u8>>,
        delivery: DeliverySemantics,
    ) -> Self {
        Self {
            address: address.into(),
            body: body.into(),
            delivery,
            metadata: ProtocolMetadata::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumerSelection {
    Address(String),
    Group { address: String, group: String },
    Queue(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionCommand {
    Connect {
        connection_id: TransportConnectionId,
    },
    Disconnect {
        connection_id: TransportConnectionId,
    },
    Publish {
        connection_id: TransportConnectionId,
        message: MessageEnvelope,
    },
    Subscribe {
        connection_id: TransportConnectionId,
        consumer: ConsumerSelection,
        delivery: DeliverySemantics,
    },
    Unsubscribe {
        connection_id: TransportConnectionId,
        consumer: ConsumerSelection,
    },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportEvent {
    Connected {
        connection_id: TransportConnectionId,
    },
    Disconnected {
        connection_id: TransportConnectionId,
    },
    MessageReceived {
        connection_id: TransportConnectionId,
        message: MessageEnvelope,
    },
    MessagePublished {
        connection_id: TransportConnectionId,
        message: MessageEnvelope,
    },
    Failure {
        connection_id: Option<TransportConnectionId>,
        message: String,
    },
    ShutdownComplete,
}

pub trait TransportPort: Send {
    fn capabilities(&self) -> &TransportCapabilities;

    fn submit(&self, command: ConnectionCommand) -> Result<(), TransportPortError>;

    fn try_next_event(&self) -> Result<Option<TransportEvent>, TransportPortError>;
}

#[derive(Debug, Error)]
pub enum TransportValueError {
    #[error("transport metadata and capability names require a namespace: {value}")]
    MissingNamespace { value: String },
    #[error("transport connection identifier is required")]
    EmptyConnectionId,
}

#[derive(Debug, Error)]
pub enum TransportPortError {
    #[error("transport port is unavailable: {message}")]
    Unavailable { message: String },
}
