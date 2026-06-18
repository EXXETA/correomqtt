use correo_mqtt::ConnectionId;
pub use correo_style::{ThemeId, ThemeSelection};
use serde::{Deserialize, Serialize};

pub type ThemeMode = ThemeSelection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayloadSyntaxKind {
    Key,
    String,
    Number,
    Keyword,
    Punctuation,
    Tag,
    Attribute,
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadSyntaxSpan {
    pub start: usize,
    pub end: usize,
    pub kind: PayloadSyntaxKind,
}

#[path = "types/workflow.rs"]
mod workflow;
pub use workflow::*;
#[path = "types/plugin_workflow.rs"]
mod plugin_workflow;
pub use plugin_workflow::*;
#[path = "types/connection_settings.rs"]
mod connection_settings;
pub use connection_settings::*;

use crate::{
    Diagnostic, GlobalSettingsSnapshot, MigrationRecoverySnapshot, PluginSurfaceSnapshot,
    ScriptSurfaceSnapshot, TransferSurfaceSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSnapshot {
    pub active_workspace: Workspace,
    pub active_connection: Option<ConnectionId>,
    pub connection_count: usize,
    pub connection_filter: String,
    pub connection_settings: ConnectionSettingsSnapshot,
    #[serde(default)]
    pub connection_settings_overlay: Option<ConnectionId>,
    #[serde(default)]
    pub connection_plugins_overlay: Option<ConnectionId>,
    pub connection_surface: ConnectionSurface,
    pub connections: Vec<ConnectionSummary>,
    pub diagnostics: Vec<Diagnostic>,
    pub global_settings: GlobalSettingsSnapshot,
    pub migration_recovery: MigrationRecoverySnapshot,
    pub plugins: PluginSurfaceSnapshot,
    pub scripts: ScriptSurfaceSnapshot,
    pub selected_connection: Option<ConnectionId>,
    pub theme_mode: ThemeMode,
    pub transfer: TransferSurfaceSnapshot,
    pub workbench: WorkbenchSnapshot,
}

impl AppSnapshot {
    pub fn empty() -> Self {
        Self {
            active_workspace: Workspace::Connections,
            active_connection: None,
            connection_count: 0,
            connection_filter: String::new(),
            connection_settings: ConnectionSettingsSnapshot::default(),
            connection_settings_overlay: None,
            connection_plugins_overlay: None,
            connection_surface: ConnectionSurface::Workbench,
            connections: Vec::new(),
            diagnostics: Vec::new(),
            global_settings: GlobalSettingsSnapshot::default(),
            migration_recovery: MigrationRecoverySnapshot::default(),
            plugins: PluginSurfaceSnapshot::default(),
            scripts: ScriptSurfaceSnapshot::default(),
            selected_connection: None,
            theme_mode: ThemeMode::System,
            transfer: TransferSurfaceSnapshot::default(),
            workbench: WorkbenchSnapshot::default(),
        }
    }

    pub fn selected_connection(&self) -> Option<&ConnectionSummary> {
        let selected = self.selected_connection?;
        self.connections
            .iter()
            .find(|connection| connection.id == selected)
    }

    pub fn filtered_connections(&self) -> Vec<&ConnectionSummary> {
        let needle = self.connection_filter.trim().to_ascii_lowercase();
        self.connections
            .iter()
            .filter(|connection| {
                needle.is_empty()
                    || connection.name.to_ascii_lowercase().contains(&needle)
                    || connection.endpoint.to_ascii_lowercase().contains(&needle)
            })
            .collect()
    }
}

impl Default for AppSnapshot {
    fn default() -> Self {
        crate::sample_snapshot(ThemeMode::System)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Workspace {
    Connections,
    ImportExport,
    Scripts,
    Plugins,
    Diagnostics,
    Settings,
    About,
}

impl Workspace {
    pub const ALL: [Self; 6] = [
        Self::Connections,
        Self::Scripts,
        Self::Plugins,
        Self::Diagnostics,
        Self::Settings,
        Self::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Connections => "Connections",
            Self::ImportExport => "Import/Export",
            Self::Scripts => "Scripting",
            Self::Plugins => "Plugins",
            Self::Diagnostics => "Diagnostics",
            Self::Settings => "Settings",
            Self::About => "About",
        }
    }

    pub fn rail_label(self) -> &'static str {
        match self {
            Self::Connections => "C",
            Self::ImportExport => "I/O",
            Self::Scripts => "S",
            Self::Plugins => "P",
            Self::Diagnostics => "D",
            Self::Settings => "G",
            Self::About => "?",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionSurface {
    Launcher,
    Workbench,
    Settings,
    Transfer,
}

impl Default for ConnectionSurface {
    fn default() -> Self {
        Self::Workbench
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

impl ConnectionState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting",
            Self::Connected => "Connected",
            Self::Reconnecting => "Reconnecting",
            Self::Error => "Error",
        }
    }

    pub fn blocks_connect(self) -> bool {
        matches!(
            self,
            Self::Connecting | Self::Connected | Self::Reconnecting
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionBadge {
    Credentials,
    Tls,
    Proxy,
    Lwt,
}

impl ConnectionBadge {
    pub fn label(self) -> &'static str {
        match self {
            Self::Credentials => "Credentials",
            Self::Tls => "TLS",
            Self::Proxy => "Tunnel",
            Self::Lwt => "LWT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectDisabledReason {
    AlreadyConnected,
    MissingHost,
    Busy,
}

impl ConnectDisabledReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::AlreadyConnected => "Already connected",
            Self::MissingHost => "Host is required",
            Self::Busy => "Connection is busy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionSummary {
    pub id: ConnectionId,
    pub name: String,
    pub endpoint: String,
    pub mqtt_version: String,
    pub badges: Vec<ConnectionBadge>,
    #[serde(default)]
    pub active_plugin_workflows: bool,
    pub state: ConnectionState,
    pub disabled_reason: Option<ConnectDisabledReason>,
    pub recent_subscriptions: usize,
    pub recent_messages: usize,
    pub last_activity: String,
}

impl ConnectionSummary {
    pub fn can_connect(&self) -> bool {
        self.disabled_reason.is_none() && !self.state.blocks_connect()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum QosLevel {
    #[default]
    Zero,
    One,
    Two,
}

impl QosLevel {
    pub const ALL: [Self; 3] = [Self::Zero, Self::One, Self::Two];

    pub fn label(self) -> &'static str {
        match self {
            Self::Zero => "QoS 0",
            Self::One => "QoS 1",
            Self::Two => "QoS 2",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkbenchTab {
    #[default]
    Publish,
    Subscribe,
}

impl WorkbenchTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Publish => "Publish",
            Self::Subscribe => "Subscribe",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageInspectorTab {
    #[default]
    Payload,
    Properties,
    Formatted,
    Diagnostics,
}

impl MessageInspectorTab {
    pub const ALL: [Self; 4] = [
        Self::Payload,
        Self::Properties,
        Self::Formatted,
        Self::Diagnostics,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Payload => "Payload",
            Self::Properties => "Properties",
            Self::Formatted => "Formatted",
            Self::Diagnostics => "Diagnostics",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkbenchSnapshot {
    pub publish: PublishPaneSnapshot,
    pub subscribe: SubscribePaneSnapshot,
    pub messages: Vec<MessageRow>,
    pub selected_message_id: Option<u32>,
    pub inspector_tab: MessageInspectorTab,
    pub detail: MessageDetailSnapshot,
    pub narrow_tab: WorkbenchTab,
    pub reconnect_status: String,
}

impl WorkbenchSnapshot {
    pub fn selected_message(&self) -> Option<&MessageRow> {
        let selected = self.selected_message_id?;
        self.messages.iter().find(|message| message.id == selected)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishPaneSnapshot {
    pub topic: String,
    pub topic_history: Vec<String>,
    pub valid: bool,
    pub qos: QosLevel,
    pub retained: bool,
    pub payload: String,
    pub validation: Vec<String>,
    pub feedback: Option<WorkflowFeedback>,
    pub history_filter: String,
    pub history: Vec<PublishHistoryRow>,
    #[serde(default)]
    pub selected_history_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishHistoryRow {
    #[serde(default)]
    pub id: u32,
    pub topic: String,
    pub timestamp: String,
    pub qos: QosLevel,
    pub retained: bool,
    #[serde(default)]
    pub payload: Vec<u8>,
    #[serde(default)]
    pub payload_preview: String,
    pub byte_size: usize,
    #[serde(default)]
    pub badges: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscribePaneSnapshot {
    pub topic: String,
    pub topic_history: Vec<String>,
    pub valid: bool,
    pub qos: QosLevel,
    pub validation: Vec<String>,
    pub feedback: Option<WorkflowFeedback>,
    pub subscriptions: Vec<SubscriptionRow>,
    pub unsubscribe_all_confirmation_count: Option<usize>,
    pub message_filter: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionRow {
    pub topic_filter: String,
    pub qos: QosLevel,
    pub message_count: usize,
    pub active: bool,
    #[serde(default = "default_subscription_messages_visible")]
    pub messages_visible: bool,
    #[serde(default)]
    pub selected: bool,
}

fn default_subscription_messages_visible() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageRow {
    pub id: u32,
    pub topic: String,
    pub timestamp: String,
    pub qos: QosLevel,
    pub retained: bool,
    pub payload: Vec<u8>,
    pub payload_preview: String,
    pub byte_size: usize,
    pub badges: Vec<String>,
    pub diagnostics: Vec<MessageDiagnosticRow>,
    pub formatted_detail: Option<FormattedMessageDetail>,
}
