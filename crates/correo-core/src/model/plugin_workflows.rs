use crate::{
    redact_sensitive, Diagnostic, FormattedMessageDetail, MessageDiagnosticRow, PluginFeedback,
    PluginHookDiagnosticEvent, PluginHookStatus, PluginStatus, PluginWorkflowEvent,
    WorkflowFeedback,
};

use super::AppModel;

impl AppModel {
    pub(super) fn select_detail_transform(&mut self, plugin_id: Option<String>) {
        self.snapshot.workbench.detail.selected_transform_plugin_id = plugin_id;
    }

    pub(super) fn select_detail_formatter(&mut self, plugin_id: Option<String>) {
        self.snapshot.workbench.detail.selected_formatter_plugin_id = plugin_id;
    }

    pub(super) fn apply_plugin_workflow_event(&mut self, event: PluginWorkflowEvent) {
        match event {
            PluginWorkflowEvent::PublishBlocked { message } => {
                let message = redact_sensitive(&message);
                self.snapshot.workbench.publish.feedback =
                    Some(WorkflowFeedback::error(message.clone()));
                self.push_diagnostic(Diagnostic::error(message));
            }
            PluginWorkflowEvent::PublishWarning { message } => {
                let message = redact_sensitive(&message);
                self.snapshot.workbench.publish.feedback =
                    Some(WorkflowFeedback::warning(message.clone()));
                self.push_diagnostic(Diagnostic::warning(message));
            }
            PluginWorkflowEvent::HookDiagnostic(diagnostic) => {
                self.record_plugin_diagnostic(diagnostic);
            }
            PluginWorkflowEvent::MessageDiagnosticsAppended {
                message_id,
                diagnostics,
            } => self.append_message_diagnostics(message_id, diagnostics),
            PluginWorkflowEvent::MessageDetailUpdated { message_id, detail } => {
                self.update_message_detail(message_id, Some(detail));
            }
            PluginWorkflowEvent::MessageDetailCleared { message_id } => {
                self.update_message_detail(message_id, None);
            }
            PluginWorkflowEvent::PluginWindowOpened(window) => self.open_plugin_window(window),
            PluginWorkflowEvent::PluginWindowRendered {
                plugin_id,
                action_id,
                connection_id,
                nodes,
            } => self.update_plugin_window(&plugin_id, &action_id, connection_id, nodes),
            PluginWorkflowEvent::PluginWindowClosed {
                plugin_id,
                action_id,
                connection_id,
            } => self.close_plugin_window(&plugin_id, &action_id, connection_id),
        }
    }

    fn open_plugin_window(&mut self, window: crate::PluginWindowRow) {
        self.close_plugin_window(&window.plugin_id, &window.action_id, window.connection_id);
        self.snapshot.plugins.open_windows.push(window);
    }

    fn update_plugin_window(
        &mut self,
        plugin_id: &str,
        action_id: &str,
        connection_id: correo_mqtt::ConnectionId,
        nodes: Vec<crate::PluginUiNode>,
    ) {
        if let Some(window) = self
            .snapshot
            .plugins
            .open_windows
            .iter_mut()
            .find(|window| window.matches(plugin_id, action_id, connection_id))
        {
            window.nodes = nodes;
        }
    }

    fn close_plugin_window(
        &mut self,
        plugin_id: &str,
        action_id: &str,
        connection_id: correo_mqtt::ConnectionId,
    ) {
        self.snapshot
            .plugins
            .open_windows
            .retain(|window| !window.matches(plugin_id, action_id, connection_id));
    }

    fn record_plugin_diagnostic(&mut self, event: PluginHookDiagnosticEvent) {
        let Some(plugin) = self
            .snapshot
            .plugins
            .plugins
            .iter_mut()
            .find(|plugin| plugin.id == event.plugin_id)
        else {
            self.push_diagnostic(Diagnostic::warning(format!(
                "Plugin diagnostic for unknown plugin {}: {}",
                event.plugin_id, event.message
            )));
            return;
        };

        let row = crate::PluginDiagnosticRow {
            id: format!("diag-{}-{}", plugin.id, plugin.diagnostics.len() + 1),
            plugin_id: plugin.id.clone(),
            severity: event.severity,
            hook: event.hook,
            message: redact_sensitive(&event.message),
            detail: redact_sensitive(&event.detail),
            occurred_at: "now".to_owned(),
        };
        plugin.diagnostics.insert(0, row);
        plugin.diagnostics.truncate(20);
        if event.mark_hook_failed {
            plugin.status = PluginStatus::HookFailed;
            if let Some(hook) = event.hook {
                if let Some(assignment) = plugin.hooks.iter_mut().find(|item| item.hook == hook) {
                    assignment.status = PluginHookStatus::Failed;
                    assignment.last_run = "now".to_owned();
                    assignment.message = redact_sensitive(&event.message);
                }
            }
        }
        self.snapshot.plugins.feedback = Some(PluginFeedback::warning(format!(
            "{} reported a plugin hook diagnostic.",
            plugin.name
        )));
    }

    fn append_message_diagnostics(
        &mut self,
        message_id: u32,
        diagnostics: Vec<MessageDiagnosticRow>,
    ) {
        let Some(message) = self
            .snapshot
            .workbench
            .messages
            .iter_mut()
            .find(|message| message.id == message_id)
        else {
            return;
        };
        message
            .diagnostics
            .extend(diagnostics.into_iter().map(|mut row| {
                row.message = redact_sensitive(&row.message);
                row
            }));
        if !message.badges.iter().any(|badge| badge == "plugin") {
            message.badges.push("plugin".to_owned());
        }
    }

    fn update_message_detail(&mut self, message_id: u32, detail: Option<FormattedMessageDetail>) {
        if let Some(message) = self
            .snapshot
            .workbench
            .messages
            .iter_mut()
            .find(|message| message.id == message_id)
        {
            message.formatted_detail = detail;
        }
    }
}
