use correo_mqtt::ConnectionId;

use crate::{PluginDiagnosticSeverity, PluginHookKind};

use super::{FormattedMessageDetail, MessageDiagnosticRow, PluginUiNode, PluginWindowRow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginWorkflowEvent {
    PublishBlocked {
        message: String,
    },
    PublishWarning {
        message: String,
    },
    HookDiagnostic(PluginHookDiagnosticEvent),
    MessageDiagnosticsAppended {
        message_id: u32,
        diagnostics: Vec<MessageDiagnosticRow>,
    },
    MessageDetailUpdated {
        message_id: u32,
        detail: FormattedMessageDetail,
    },
    MessageDetailCleared {
        message_id: u32,
    },
    PluginWindowOpened(PluginWindowRow),
    PluginWindowRendered {
        plugin_id: String,
        action_id: String,
        connection_id: ConnectionId,
        nodes: Vec<PluginUiNode>,
    },
    PluginWindowClosed {
        plugin_id: String,
        action_id: String,
        connection_id: ConnectionId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginHookDiagnosticEvent {
    pub plugin_id: String,
    pub hook: Option<PluginHookKind>,
    pub severity: PluginDiagnosticSeverity,
    pub message: String,
    pub detail: String,
    pub mark_hook_failed: bool,
}
