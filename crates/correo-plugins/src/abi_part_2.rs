#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomingTransportMessageTransformRequest {
    pub abi_version: u16,
    pub context: HookContextDto,
    #[serde(default)]
    pub config: Value,
    pub input: TransportHookInputDto,
}

impl VersionedDto for IncomingTransportMessageTransformRequest {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportMessageValidatorRequest {
    pub abi_version: u16,
    pub context: HookContextDto,
    #[serde(default)]
    pub config: Value,
    pub input: TransportHookInputDto,
}

impl VersionedDto for TransportMessageValidatorRequest {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TransportMessageTransformOutcomeDto {
    Unchanged,
    Replace { input: TransportHookInputDto },
    Drop { reason: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutgoingTransportMessageTransformResponse {
    pub abi_version: u16,
    pub outcome: TransportMessageTransformOutcomeDto,
}

impl VersionedDto for OutgoingTransportMessageTransformResponse {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomingTransportMessageTransformResponse {
    pub abi_version: u16,
    pub outcome: TransportMessageTransformOutcomeDto,
}

impl VersionedDto for IncomingTransportMessageTransformResponse {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportMessageValidatorResponse {
    pub abi_version: u16,
    pub result: ValidationResultDto,
}

impl VersionedDto for TransportMessageValidatorResponse {
    fn abi_version(&self) -> u16 {
        self.abi_version
    }
}
