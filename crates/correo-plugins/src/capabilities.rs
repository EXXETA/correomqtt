use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookKind {
    OutgoingMessageTransform,
    IncomingMessageTransform,
    MessageValidator,
    DetailByteTransform,
    DetailFormatter,
    PayloadHighlighter,
}

impl HookKind {
    pub const ALL: [Self; 6] = [
        Self::OutgoingMessageTransform,
        Self::IncomingMessageTransform,
        Self::MessageValidator,
        Self::DetailByteTransform,
        Self::DetailFormatter,
        Self::PayloadHighlighter,
    ];

    /// Hooks that go through the WASM dispatch ABI. `PayloadHighlighter` is a
    /// built-in host capability without a WASM invocation and therefore has
    /// no noop fixture.
    pub const WASM_DISPATCHED: [Self; 5] = [
        Self::OutgoingMessageTransform,
        Self::IncomingMessageTransform,
        Self::MessageValidator,
        Self::DetailByteTransform,
        Self::DetailFormatter,
    ];

    pub fn fixture_file_name(self) -> &'static str {
        match self {
            Self::OutgoingMessageTransform => "outgoing_message_transform.json",
            Self::IncomingMessageTransform => "incoming_message_transform.json",
            Self::MessageValidator => "message_validator.json",
            Self::DetailByteTransform => "detail_byte_transform.json",
            Self::DetailFormatter => "detail_formatter.json",
            Self::PayloadHighlighter => "payload_highlighter.json",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CapabilityGrants {
    #[serde(default)]
    pub hooks: Vec<HookKind>,
    #[serde(default)]
    pub host: HostCapabilityGrants,
}

impl CapabilityGrants {
    pub fn grants_hook(&self, hook: HookKind) -> bool {
        self.hooks.contains(&hook)
    }

    pub fn grants_host_surface(&self, surface: HostSurface) -> bool {
        self.host.grants(surface)
    }

    pub fn ensure_hook(&self, hook: HookKind) -> Result<(), CapabilityError> {
        if self.grants_hook(hook) {
            Ok(())
        } else {
            Err(CapabilityError::HookDenied(hook))
        }
    }

    pub fn ensure_host_surface(&self, surface: HostSurface) -> Result<(), CapabilityError> {
        if self.grants_host_surface(surface) {
            Ok(())
        } else {
            Err(CapabilityError::HostSurfaceDenied(surface))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HostCapabilityGrants {
    #[serde(default)]
    pub filesystem: bool,
    #[serde(default)]
    pub message_save: bool,
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub secrets: bool,
    #[serde(default)]
    pub mqtt: bool,
    #[serde(default)]
    pub ui: bool,
}

impl HostCapabilityGrants {
    pub fn grants(&self, surface: HostSurface) -> bool {
        match surface {
            HostSurface::Filesystem => self.filesystem,
            HostSurface::MessageSave => self.message_save,
            HostSurface::Network => self.network,
            HostSurface::Secrets => self.secrets,
            HostSurface::Mqtt => self.mqtt,
            HostSurface::Ui => self.ui,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostSurface {
    Filesystem,
    MessageSave,
    Network,
    Secrets,
    Mqtt,
    Ui,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CapabilityError {
    #[error("plugin hook capability is not granted: {0:?}")]
    HookDenied(HookKind),
    #[error("plugin host surface is not granted: {0:?}")]
    HostSurfaceDenied(HostSurface),
}
