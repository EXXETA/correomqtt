use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const ABI_VERSION: u16 = 1;

pub trait VersionedDto {
    fn abi_version(&self) -> u16;
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HookContextDto {
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub invocation_id: Option<String>,
    #[serde(default)]
    pub subscription_topic: Option<String>,
    #[serde(default)]
    pub timestamp_unix_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageDto {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: QosDto,
    pub retained: bool,
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
}

impl MessageDto {
    pub fn new(topic: impl Into<String>, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
            qos: QosDto::AtMostOnce,
            retained: false,
            properties: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QosDto {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutgoingMessageTransformRequest {
    pub abi_version: u16,
    pub context: HookContextDto,
    #[serde(default)]
    pub config: Value,
    pub message: MessageDto,
}

impl OutgoingMessageTransformRequest {
    pub fn new(message: MessageDto) -> Self {
        Self {
            abi_version: ABI_VERSION,
            context: HookContextDto::default(),
            config: Value::Null,
            message,
        }
    }
}

impl VersionedDto for OutgoingMessageTransformRequest {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutgoingMessageTransformResponse {
    pub abi_version: u16,
    pub outcome: MessageTransformOutcomeDto,
}

impl OutgoingMessageTransformResponse {
    pub fn unchanged() -> Self {
        Self {
            abi_version: ABI_VERSION,
            outcome: MessageTransformOutcomeDto::Unchanged,
        }
    }
}

impl VersionedDto for OutgoingMessageTransformResponse {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomingMessageTransformRequest {
    pub abi_version: u16,
    pub context: HookContextDto,
    #[serde(default)]
    pub config: Value,
    pub message: MessageDto,
}

impl IncomingMessageTransformRequest {
    pub fn new(message: MessageDto) -> Self {
        Self {
            abi_version: ABI_VERSION,
            context: HookContextDto::default(),
            config: Value::Null,
            message,
        }
    }
}

impl VersionedDto for IncomingMessageTransformRequest {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomingMessageTransformResponse {
    pub abi_version: u16,
    pub outcome: MessageTransformOutcomeDto,
}

impl IncomingMessageTransformResponse {
    pub fn unchanged() -> Self {
        Self {
            abi_version: ABI_VERSION,
            outcome: MessageTransformOutcomeDto::Unchanged,
        }
    }
}

impl VersionedDto for IncomingMessageTransformResponse {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MessageTransformOutcomeDto {
    Unchanged,
    Replace { message: MessageDto },
    Drop { reason: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageValidatorRequest {
    pub abi_version: u16,
    pub context: HookContextDto,
    #[serde(default)]
    pub config: Value,
    pub message: MessageDto,
}

impl MessageValidatorRequest {
    pub fn new(message: MessageDto) -> Self {
        Self {
            abi_version: ABI_VERSION,
            context: HookContextDto::default(),
            config: Value::Null,
            message,
        }
    }
}

impl VersionedDto for MessageValidatorRequest {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageValidatorResponse {
    pub abi_version: u16,
    pub result: ValidationResultDto,
}

impl MessageValidatorResponse {
    pub fn valid() -> Self {
        Self {
            abi_version: ABI_VERSION,
            result: ValidationResultDto::Valid,
        }
    }
}

impl VersionedDto for MessageValidatorResponse {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ValidationResultDto {
    Valid,
    Invalid { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailByteTransformRequest {
    pub abi_version: u16,
    pub context: HookContextDto,
    #[serde(default)]
    pub config: Value,
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub content_type: Option<String>,
}

impl DetailByteTransformRequest {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            abi_version: ABI_VERSION,
            context: HookContextDto::default(),
            config: Value::Null,
            bytes: bytes.into(),
            content_type: None,
        }
    }
}

impl VersionedDto for DetailByteTransformRequest {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetailByteTransformResponse {
    pub abi_version: u16,
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub host_actions: Vec<HostActionDto>,
}

impl DetailByteTransformResponse {
    pub fn unchanged(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            abi_version: ABI_VERSION,
            bytes: bytes.into(),
            content_type: None,
            host_actions: Vec::new(),
        }
    }
}

impl VersionedDto for DetailByteTransformResponse {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostActionDto {
    SavePayload(SavePayloadActionDto),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavePayloadActionDto {
    pub suggested_file_name: String,
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailFormatterRequest {
    pub abi_version: u16,
    pub context: HookContextDto,
    #[serde(default)]
    pub config: Value,
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub content_type: Option<String>,
}

impl DetailFormatterRequest {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            abi_version: ABI_VERSION,
            context: HookContextDto::default(),
            config: Value::Null,
            bytes: bytes.into(),
            content_type: None,
        }
    }
}

impl VersionedDto for DetailFormatterRequest {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetailFormatterResponse {
    pub abi_version: u16,
    pub output: FormattedDetailDto,
}

impl DetailFormatterResponse {
    pub fn plain_text(text: impl Into<String>) -> Self {
        Self {
            abi_version: ABI_VERSION,
            output: FormattedDetailDto {
                format: DetailFormatDto::PlainText,
                text: text.into(),
                diagnostics: Vec::new(),
            },
        }
    }
}

impl VersionedDto for DetailFormatterResponse {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormattedDetailDto {
    pub format: DetailFormatDto,
    pub text: String,
    #[serde(default)]
    pub diagnostics: Vec<HookDiagnosticDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetailFormatDto {
    PlainText,
    Json,
    Xml,
    Hex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookDiagnosticDto {
    pub severity: HookDiagnosticSeverityDto,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookDiagnosticSeverityDto {
    Info,
    Warning,
    Error,
}

pub const ABI_VERSION_V2: u16 = 2;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct NamespacedNameDto(String);

impl NamespacedNameDto {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value
            .split_once('.')
            .is_some_and(|(namespace, name)| !namespace.is_empty() && !name.is_empty())
        {
            Ok(Self(value))
        } else {
            Err(format!("transport value requires a namespace: {value}"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for NamespacedNameDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|value| Self::new(value).map_err(serde::de::Error::custom))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryGuaranteeDto {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportMessageDto {
    pub address: String,
    pub body: Vec<u8>,
    pub delivery: DeliveryGuaranteeDto,
    #[serde(default)]
    pub metadata: BTreeMap<NamespacedNameDto, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConsumerSelectionDto {
    Address { address: String },
    Group { address: String, group: String },
    Queue { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportHookInputDto {
    pub message: TransportMessageDto,
    #[serde(default)]
    pub consumer: Option<ConsumerSelectionDto>,
    #[serde(default)]
    pub capabilities: std::collections::BTreeSet<NamespacedNameDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutgoingTransportMessageTransformRequest {
    pub abi_version: u16,
    pub context: HookContextDto,
    #[serde(default)]
    pub config: Value,
    pub input: TransportHookInputDto,
}

impl VersionedDto for OutgoingTransportMessageTransformRequest {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

