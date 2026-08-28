use std::io::{Read, Write};

use base64::Engine;
use correo_mqtt::{PublishRequest, Subscription, UnsubscribeRequest};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde_json::Value;

use crate::{
    AppCommand, AppEvent, ConnectionPluginDirection, ConnectionPluginWorkflow,
    ConnectionPluginWorkflowKind, DetailBytesOutput, FormattedMessageDetail, MessageDiagnosticRow,
    MessageInspectorTab, MqttCommand, MqttCommandBuildError, MqttEvent, PluginDiagnosticSeverity,
    PluginHookCall, PluginHookDiagnosticEvent, PluginHookError, PluginHookInput, PluginHookKind,
    PluginHookOutput, PluginHookStatus, PluginHostAction, PluginOpenWindow, PluginStatus,
    PluginValidation, PluginWindowCloseRequest, PluginWindowMessage, PluginWindowRenderRequest,
    PluginWindowRow, PluginWorkflowEvent,
};

use super::plugin_helpers::*;
use super::AppRuntime;

pub(super) enum IncomingPluginDispatch {
    NotApplicable,
    Queued,
    Continue {
        event: MqttEvent,
        diagnostics: Vec<MessageDiagnosticRow>,
    },
    Rejected,
}

impl AppRuntime {
    pub(super) fn apply_plugin_connection_command(&mut self, command: &AppCommand) {
        match command {
            AppCommand::InvokeConnectionPluginAction {
                plugin_id,
                action_id,
                connection_id,
            } => {
                let request = crate::PluginConnectionActionRequest {
                    plugin_id: plugin_id.clone(),
                    action_id: action_id.clone(),
                    connection_id: *connection_id,
                    connection_name: self
                        .model
                        .snapshot()
                        .connections
                        .iter()
                        .find(|connection| connection.id == *connection_id)
                        .map(|connection| connection.name.clone())
                        .unwrap_or_default(),
                };
                match self.plugin_hooks.connection_action(request) {
                    Ok(response) => {
                        self.forward_plugin_host_actions(response.host_actions, false);
                        if let Some(window) = response.open_window {
                            self.open_plugin_window(window);
                        }
                    }
                    Err(error) => self.emit_plugin_action_error(plugin_id, error),
                }
            }
            AppCommand::ClosePluginWindow {
                plugin_id,
                action_id,
                connection_id,
            } => {
                let request = PluginWindowCloseRequest {
                    plugin_id: plugin_id.clone(),
                    action_id: action_id.clone(),
                    connection_id: *connection_id,
                };
                match self.plugin_hooks.close_window(request) {
                    Ok(response) => self.forward_plugin_host_actions(response.host_actions, false),
                    Err(error) => self.emit_plugin_action_error(plugin_id, error),
                }
                self.emit_plugin_event(PluginWorkflowEvent::PluginWindowClosed {
                    plugin_id: plugin_id.clone(),
                    action_id: action_id.clone(),
                    connection_id: *connection_id,
                });
            }
            _ => {}
        }
    }

    pub(super) fn mqtt_commands_for_app_command_with_plugins(
        &self,
        command: &AppCommand,
    ) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
        let commands = self.model.mqtt_commands_for_app_command(command)?;

        let mut transformed = Vec::new();
        for command in commands {
            match command {
                MqttCommand::Publish {
                    connection_id,
                    request,
                    diagnostics,
                } => {
                    if let Some((request, diagnostics)) =
                        self.apply_publish_hooks(connection_id, request, diagnostics)
                    {
                        transformed.push(MqttCommand::Publish {
                            connection_id,
                            request,
                            diagnostics,
                        });
                    }
                }
                command => transformed.push(command),
            }
        }
        Ok(transformed)
    }

    pub(super) fn refresh_plugin_windows(&self) {
        let windows = self.model.snapshot().plugins.open_windows.clone();
        for window in windows {
            self.render_plugin_window(&window);
        }
    }

    fn open_plugin_window(&self, window: PluginOpenWindow) {
        let row = PluginWindowRow {
            plugin_id: window.plugin_id,
            action_id: window.action_id,
            connection_id: window.connection_id,
            title: window.title,
            message_filter_prefix: window.message_filter_prefix,
            latest_per_topic: window.latest_per_topic,
            nodes: Vec::new(),
        };
        self.emit_plugin_event(PluginWorkflowEvent::PluginWindowOpened(row.clone()));
        self.render_plugin_window(&row);
    }

    fn render_plugin_window(&self, window: &PluginWindowRow) {
        let messages = if self.model.snapshot().selected_connection == Some(window.connection_id) {
            plugin_window_messages(window, &self.model.snapshot().workbench.messages)
        } else {
            Vec::new()
        };
        let broker = self
            .model
            .snapshot()
            .connections
            .iter()
            .find(|connection| connection.id == window.connection_id)
            .map(|connection| connection.endpoint.clone())
            .unwrap_or_default();
        let request = PluginWindowRenderRequest {
            plugin_id: window.plugin_id.clone(),
            action_id: window.action_id.clone(),
            connection_id: window.connection_id,
            broker,
            messages,
        };
        match self.plugin_hooks.render_window(request) {
            Ok(response) => self.emit_plugin_event(PluginWorkflowEvent::PluginWindowRendered {
                plugin_id: window.plugin_id.clone(),
                action_id: window.action_id.clone(),
                connection_id: window.connection_id,
                nodes: response.nodes,
            }),
            Err(error) => self.emit_plugin_action_error(&window.plugin_id, error),
        }
    }

    fn forward_plugin_host_actions(
        &mut self,
        actions: Vec<PluginHostAction>,
        allow_save_payload: bool,
    ) {
        for action in actions {
            match action {
                PluginHostAction::SavePayload(payload) if allow_save_payload => {
                    if self.pending_plugin_save_payloads.len()
                        == super::PLUGIN_SAVE_PAYLOAD_CAPACITY
                    {
                        let _ = self.event_sender().emit(AppEvent::DiagnosticRaised(
                            crate::Diagnostic::error("Plugin save request was rejected because another save is awaiting confirmation."),
                        ));
                    } else {
                        self.pending_plugin_save_payloads.push_back(payload);
                    }
                }
                action => match plugin_host_action_to_mqtt(action) {
                    Ok(command) => self.forward_plugin_mqtt_command(command),
                    Err(error) => {
                        let _ = self
                            .event_sender()
                            .emit(AppEvent::DiagnosticRaised(crate::Diagnostic::error(error)));
                    }
                },
            }
        }
    }

    fn forward_plugin_mqtt_command(&self, command: MqttCommand) {
        let Some(service) = &self.mqtt_service else {
            let _ =
                self.event_sender()
                    .emit(AppEvent::DiagnosticRaised(crate::Diagnostic::warning(
                        "MQTT service is not running.",
                    )));
            return;
        };
        if let Err(error) = service.command_sender().send(command) {
            let _ = self
                .event_sender()
                .emit(AppEvent::DiagnosticRaised(crate::Diagnostic::error(
                    error.to_string(),
                )));
        }
    }

    fn emit_plugin_action_error(&self, plugin_id: &str, error: PluginHookError) {
        self.emit_plugin_event(PluginWorkflowEvent::HookDiagnostic(
            PluginHookDiagnosticEvent {
                plugin_id: plugin_id.to_owned(),
                hook: None,
                severity: PluginDiagnosticSeverity::Error,
                message: "Plugin connection action failed.".to_owned(),
                detail: error.to_string(),
                mark_hook_failed: false,
            },
        ));
    }

    pub(super) fn queue_incoming_hook_job(
        &self,
        event: &MqttEvent,
    ) -> IncomingPluginDispatch {
        let Some(worker) = &self.incoming_plugin_worker else {
            return IncomingPluginDispatch::NotApplicable;
        };
        let MqttEvent::IncomingMessage(message) = event else {
            return IncomingPluginDispatch::NotApplicable;
        };
        let message = message.clone();
        let original_topic = message.topic.as_str().to_owned();
        let mut plugin_message = plugin_message_from_incoming(&message);
        let diagnostics = self.apply_connection_workflows(
            message.connection_id,
            &mut plugin_message,
            ConnectionPluginDirection::Incoming,
        );
        let validator_topic = plugin_message.topic.clone();
        let Ok(message) = incoming_from_plugin_message(message, plugin_message) else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(crate::Diagnostic::error(
                    "Incoming plugin workflow returned an invalid topic.",
                )));
            return IncomingPluginDispatch::Rejected;
        };
        let mut hooks = self.active_topic_hooks(PluginHookKind::IncomingTransform, &original_topic);
        hooks.extend(self.active_topic_hooks(PluginHookKind::Validator, &validator_topic));
        match worker.enqueue(message, hooks, diagnostics) {
            Ok(()) => IncomingPluginDispatch::Queued,
            Err(error) => {
                let (message, mut diagnostics, detail) = match error {
                    super::incoming_plugins::IncomingPluginQueueError::Full {
                        message,
                        diagnostics,
                    } => (
                        message,
                        diagnostics,
                        "Incoming plugin queue is full; message continued without topic plugin hooks.",
                    ),
                    super::incoming_plugins::IncomingPluginQueueError::Disconnected {
                        message,
                        diagnostics,
                    } => (
                        message,
                        diagnostics,
                        "Incoming plugin worker is unavailable; message continued without topic plugin hooks.",
                    ),
                };
                diagnostics.push(MessageDiagnosticRow {
                    severity: PluginDiagnosticSeverity::Warning,
                    hook: None,
                    plugin_id: None,
                    message: detail.to_owned(),
                });
                let _ = self
                    .event_sender
                    .emit(AppEvent::DiagnosticRaised(crate::Diagnostic::warning(detail)));
                IncomingPluginDispatch::Continue {
                    event: MqttEvent::IncomingMessage(message),
                    diagnostics,
                }
            }
        }
    }

    pub(super) fn try_recv_incoming_hook_result(
        &self,
    ) -> Option<super::incoming_plugins::IncomingPluginResult> {
        self.incoming_plugin_worker
            .as_ref()
            .and_then(super::incoming_plugins::IncomingPluginWorker::try_recv)
    }

    pub(super) fn apply_incoming_hooks(
        &self,
        event: MqttEvent,
    ) -> Option<(MqttEvent, Vec<MessageDiagnosticRow>)> {
        let MqttEvent::IncomingMessage(message) = event else {
            return Some((event, Vec::new()));
        };
        let topic = message.topic.as_str().to_owned();
        let mut plugin_message = plugin_message_from_incoming(&message);
        let mut diagnostics = self.apply_connection_workflows(
            message.connection_id,
            &mut plugin_message,
            ConnectionPluginDirection::Incoming,
        );
        let retained_when_absent = plugin_message.retained;
        let mut transport = plugin_transport_message_from_incoming(&message, plugin_message);
        let transport_capabilities = transport.capabilities.clone();

        for hook in self.active_topic_hooks(PluginHookKind::IncomingTransform, &topic) {
            let Some(config) = self.parse_hook_config(&hook, false) else {
                continue;
            };
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::TransportMessage(transport.clone()),
            };
            match self.plugin_hooks.execute(call) {
                Ok(PluginHookOutput::TransportMessageTransform(
                    crate::TransportMessageTransform::Unchanged,
                )) => {}
                Ok(PluginHookOutput::TransportMessageTransform(
                    crate::TransportMessageTransform::Replace(replacement),
                )) => {
                    transport = replacement;
                }
                Ok(PluginHookOutput::TransportMessageTransform(
                    crate::TransportMessageTransform::Drop { reason },
                )) => {
                    self.emit_hook_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Warning,
                        "Incoming message dropped by plugin transform.",
                        reason.unwrap_or_else(|| {
                            "Plugin requested the message be dropped.".to_owned()
                        }),
                        false,
                    );
                    return None;
                }
                Ok(output) => {
                    let detail = format!("Unexpected plugin output: {output:?}");
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        &detail,
                    ));
                    self.emit_hook_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        "Incoming transform returned an incompatible result.",
                        detail,
                        true,
                    );
                }
                Err(error) => {
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        &error.to_string(),
                    ));
                    self.emit_hook_error(&hook, "Incoming transform failed.", error, true);
                }
            }
        }

        for hook in self.active_topic_hooks(PluginHookKind::Validator, &transport.message.address) {
            let Some(config) = self.parse_hook_config(&hook, false) else {
                continue;
            };
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::TransportMessage(transport.clone()),
            };
            match self.plugin_hooks.execute(call) {
                Ok(PluginHookOutput::Validation(PluginValidation::Valid)) => {
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Info,
                        "Validation passed",
                    ));
                }
                Ok(PluginHookOutput::Validation(PluginValidation::Warning { message })) => {
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Warning,
                        &message,
                    ));
                }
                Ok(PluginHookOutput::Validation(PluginValidation::Block { message })) => {
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        &message,
                    ));
                }
                Ok(output) => {
                    let detail = format!("Unexpected plugin output: {output:?}");
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        &detail,
                    ));
                    self.emit_hook_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        "Incoming validator returned an incompatible result.",
                        detail,
                        true,
                    );
                }
                Err(error) => {
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        &error.to_string(),
                    ));
                    self.emit_hook_error(&hook, "Incoming validator failed.", error, true);
                }
            }
        }

        let plugin_message = match plugin_message_from_transport(
            transport,
            retained_when_absent,
            &transport_capabilities,
        ) {
            Ok(message) => message,
            Err(error) => {
                self.emit_hook_diagnostic(
                    &ActiveHook {
                        plugin_id: "plugin-workflow".to_owned(),
                        hook: PluginHookKind::IncomingTransform,
                        target: topic.clone(),
                        config_json: "{}".to_owned(),
                    },
                    PluginDiagnosticSeverity::Error,
                    "Incoming transform returned an invalid transport message.",
                    error,
                    false,
                );
                return None;
            }
        };
        match incoming_from_plugin_message(message, plugin_message) {
            Ok(message) => Some((MqttEvent::IncomingMessage(message), diagnostics)),
            Err(error) => {
                self.emit_hook_diagnostic(
                    &ActiveHook {
                        plugin_id: "plugin-workflow".to_owned(),
                        hook: PluginHookKind::IncomingTransform,
                        target: topic,
                        config_json: "{}".to_owned(),
                    },
                    PluginDiagnosticSeverity::Error,
                    "Incoming transform returned an invalid topic.",
                    error,
                    false,
                );
                None
            }
        }
    }

}

