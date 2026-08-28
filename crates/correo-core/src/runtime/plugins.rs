use std::io::{Read, Write};

use base64::Engine;
use correo_mqtt::{PublishRequest, Subscription, UnsubscribeRequest};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde_json::Value;

use crate::{
    AppCommand, AppEvent, ConnectionPluginDirection, ConnectionPluginWorkflow,
    ConnectionPluginWorkflowKind, DetailBytesOutput, FormattedMessageDetail, MessageDiagnosticRow,
    MessageInspectorTab, MessageTransform, MqttCommand, MqttCommandBuildError, MqttEvent,
    PluginDiagnosticSeverity, PluginHookCall, PluginHookDiagnosticEvent, PluginHookError,
    PluginHookInput, PluginHookKind, PluginHookOutput, PluginHookStatus, PluginHostAction,
    PluginOpenWindow, PluginStatus, PluginValidation, PluginWindowCloseRequest,
    PluginWindowMessage, PluginWindowRenderRequest, PluginWindowRow, PluginWorkflowEvent,
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
    pub(super) fn apply_plugin_connection_command(&self, command: &AppCommand) {
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
                        self.forward_plugin_host_actions(response.host_actions);
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
                    Ok(response) => self.forward_plugin_host_actions(response.host_actions),
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

    fn forward_plugin_host_actions(&self, actions: Vec<PluginHostAction>) {
        for action in actions {
            match plugin_host_action_to_mqtt(action) {
                Ok(command) => self.forward_plugin_mqtt_command(command),
                Err(error) => {
                    let _ = self
                        .event_sender()
                        .emit(AppEvent::DiagnosticRaised(crate::Diagnostic::error(error)));
                }
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

    pub(super) fn queue_incoming_hook_job(&self, event: &MqttEvent) -> IncomingPluginDispatch {
        let Some(worker) = &self.incoming_plugin_worker else {
            return IncomingPluginDispatch::NotApplicable;
        };
        let MqttEvent::IncomingMessage(message) = event else {
            return IncomingPluginDispatch::NotApplicable;
        };
        let original_topic = message.topic.as_str().to_owned();
        let mut plugin_message = plugin_message_from_incoming(message);
        let diagnostics = self.apply_connection_workflows(
            message.connection_id,
            &mut plugin_message,
            ConnectionPluginDirection::Incoming,
        );
        let message = match incoming_from_plugin_message(message.clone(), plugin_message.clone()) {
            Ok(message) => message,
            Err(error) => {
                self.emit_hook_diagnostic(
                    &ActiveHook {
                        plugin_id: "plugin-workflow".to_owned(),
                        hook: PluginHookKind::IncomingTransform,
                        target: original_topic,
                        config_json: "{}".to_owned(),
                    },
                    PluginDiagnosticSeverity::Error,
                    "Incoming transform returned an invalid topic.",
                    error,
                    false,
                );
                return IncomingPluginDispatch::Rejected;
            }
        };
        let transforms =
            self.active_topic_hooks(PluginHookKind::IncomingTransform, &original_topic);
        let validators = self.active_topic_hooks(PluginHookKind::Validator, &plugin_message.topic);
        let requires_validation = !validators.is_empty();
        match worker.enqueue(message, transforms, validators, diagnostics) {
            Ok(()) => IncomingPluginDispatch::Queued,
            Err(error) => {
                let detail = error.detail();
                let job = error.into_job();
                if requires_validation {
                    let _ = self.event_sender.emit(AppEvent::DiagnosticRaised(
                        crate::Diagnostic::error(format!(
                            "Incoming message rejected because the plugin validator cannot run: {detail}."
                        )),
                    ));
                    return IncomingPluginDispatch::Rejected;
                }
                let detail = format!("{detail}; message continued without topic plugin hooks.");
                let mut diagnostics = job.diagnostics;
                diagnostics.push(MessageDiagnosticRow {
                    severity: PluginDiagnosticSeverity::Warning,
                    hook: None,
                    plugin_id: None,
                    message: detail.clone(),
                });
                let _ =
                    self.event_sender
                        .emit(AppEvent::DiagnosticRaised(crate::Diagnostic::warning(
                            detail,
                        )));
                IncomingPluginDispatch::Continue {
                    event: MqttEvent::IncomingMessage(job.message),
                    diagnostics,
                }
            }
        }
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

        for hook in self.active_topic_hooks(PluginHookKind::IncomingTransform, &topic) {
            let Some(config) = self.parse_hook_config(&hook, false) else {
                continue;
            };
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::Message(plugin_message.clone()),
            };
            match self.plugin_hooks.execute(call) {
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Unchanged)) => {}
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Replace(message))) => {
                    plugin_message = message;
                }
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Drop { reason })) => {
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

        for hook in self.active_topic_hooks(PluginHookKind::Validator, &plugin_message.topic) {
            let Some(config) = self.parse_hook_config(&hook, false) else {
                continue;
            };
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::Message(plugin_message.clone()),
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

    pub(super) fn append_incoming_diagnostics(&self, diagnostics: Vec<MessageDiagnosticRow>) {
        if diagnostics.is_empty() {
            return;
        }
        let Some(message_id) = self.model.snapshot().workbench.selected_message_id else {
            return;
        };
        self.emit_plugin_event(PluginWorkflowEvent::MessageDiagnosticsAppended {
            message_id,
            diagnostics,
        });
    }

    pub(super) fn should_refresh_detail_for_command(&self, command: &AppCommand) -> bool {
        matches!(
            command,
            AppCommand::SelectMessage(_)
                | AppCommand::SelectInspectorTab(MessageInspectorTab::Formatted)
                | AppCommand::SelectDetailTransform(_)
                | AppCommand::SelectDetailFormatter(_)
                | AppCommand::SetPluginEnabled { .. }
                | AppCommand::SetPluginHookEnabled { .. }
                | AppCommand::RefreshMessageDetail
        )
    }

    pub(super) fn refresh_message_detail(&self) {
        let snapshot = self.model.snapshot();
        let Some(message) = snapshot.workbench.selected_message() else {
            return;
        };
        let message_id = message.id;
        let mut bytes = message.payload.clone();
        let mut content_type = None;
        let mut diagnostics = message.diagnostics.clone();

        if let Some(plugin_id) = &snapshot.workbench.detail.selected_transform_plugin_id {
            if let Some(hook) =
                self.selected_detail_hook(plugin_id, PluginHookKind::DetailTransform)
            {
                match self.run_detail_transform(&hook, bytes.clone(), content_type.clone()) {
                    Ok(output) => {
                        bytes = output.bytes;
                        content_type = output.content_type;
                    }
                    Err(error) => {
                        diagnostics.push(message_diagnostic(
                            &hook,
                            PluginDiagnosticSeverity::Error,
                            &error.to_string(),
                        ));
                        self.emit_hook_error(&hook, "Detail byte transform failed.", error, false);
                    }
                }
            }
        }

        let detail =
            if let Some(plugin_id) = &snapshot.workbench.detail.selected_formatter_plugin_id {
                if let Some(hook) =
                    self.selected_detail_hook(plugin_id, PluginHookKind::DetailFormatter)
                {
                    match self.run_detail_formatter(&hook, bytes.clone(), content_type.clone()) {
                        Ok(detail) => detail,
                        Err(error) => {
                            diagnostics.push(message_diagnostic(
                                &hook,
                                PluginDiagnosticSeverity::Error,
                                &error.to_string(),
                            ));
                            self.emit_hook_error(&hook, "Detail formatter failed.", error, false);
                            plain_detail(bytes, content_type, diagnostics.clone())
                        }
                    }
                } else {
                    plain_detail(bytes, content_type, diagnostics.clone())
                }
            } else {
                plain_detail(bytes, content_type, diagnostics.clone())
            };
        self.emit_plugin_event(PluginWorkflowEvent::MessageDetailUpdated { message_id, detail });
    }

    fn apply_publish_hooks(
        &self,
        connection_id: correo_mqtt::ConnectionId,
        request: PublishRequest,
        mut diagnostics: Vec<MessageDiagnosticRow>,
    ) -> Option<(PublishRequest, Vec<MessageDiagnosticRow>)> {
        let mut message = plugin_message_from_publish(&request);
        let topic = message.topic.clone();
        diagnostics.extend(self.apply_connection_workflows(
            connection_id,
            &mut message,
            ConnectionPluginDirection::Outgoing,
        ));
        if !diagnostics.is_empty() {
            self.emit_plugin_event(PluginWorkflowEvent::PublishWarning {
                message: diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.message.clone())
                    .collect::<Vec<_>>()
                    .join("; "),
            });
        }

        for hook in self.active_topic_hooks(PluginHookKind::Validator, &topic) {
            let config = self.parse_hook_config(&hook, true)?;
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::Message(message.clone()),
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
                    self.emit_plugin_event(PluginWorkflowEvent::PublishWarning { message });
                }
                Ok(PluginHookOutput::Validation(PluginValidation::Block { message })) => {
                    self.emit_hook_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        "Validator blocked publish.",
                        message.clone(),
                        false,
                    );
                    self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
                    return None;
                }
                Ok(output) => {
                    self.block_publish_for_output(&hook, output);
                    return None;
                }
                Err(error) => {
                    self.block_publish_for_error(&hook, error);
                    return None;
                }
            }
        }

        for hook in self.active_topic_hooks(PluginHookKind::OutgoingTransform, &topic) {
            let config = self.parse_hook_config(&hook, true)?;
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::Message(message.clone()),
            };
            match self.plugin_hooks.execute(call) {
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Unchanged)) => {}
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Replace(replacement))) => {
                    message = replacement;
                }
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Drop { reason })) => {
                    let message = reason
                        .unwrap_or_else(|| "Outgoing transform rejected the publish.".to_owned());
                    self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
                    return None;
                }
                Ok(output) => {
                    self.block_publish_for_output(&hook, output);
                    return None;
                }
                Err(error) => {
                    self.block_publish_for_error(&hook, error);
                    return None;
                }
            }
        }

        PublishRequest::new(
            message.topic.as_str(),
            message.payload,
            mqtt_qos(message.qos),
            message.retained,
        )
        .map_err(|error| error.to_report().message)
        .map_err(|message| {
            self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
        })
        .ok()
        .map(|request| (request, diagnostics))
    }

    fn run_detail_transform(
        &self,
        hook: &ActiveHook,
        bytes: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<DetailBytesOutput, PluginHookError> {
        let Some(config) = self.parse_hook_config(hook, false) else {
            return Err(PluginHookError::failed(
                "detail transform config is invalid",
            ));
        };
        let call = PluginHookCall {
            plugin_id: hook.plugin_id.clone(),
            hook: hook.hook,
            target: hook.target.clone(),
            config,
            input: PluginHookInput::DetailBytes {
                bytes,
                content_type,
            },
        };
        match self.plugin_hooks.execute(call)? {
            PluginHookOutput::DetailBytes(output) => Ok(output),
            output => Err(PluginHookError::failed(format!(
                "detail transform returned incompatible output: {output:?}"
            ))),
        }
    }

    fn apply_connection_workflows(
        &self,
        connection_id: correo_mqtt::ConnectionId,
        message: &mut crate::PluginMessage,
        direction: ConnectionPluginDirection,
    ) -> Vec<MessageDiagnosticRow> {
        let Some(settings) = self.model.connection_settings_for(connection_id) else {
            return Vec::new();
        };
        let mut diagnostics = Vec::new();
        for workflow in &settings.plugin_workflows {
            if !workflow.enabled
                || !workflow.available
                || !workflow_direction_matches(workflow.direction, direction)
                || !topic_matches_filter(&message.topic, &workflow.topic_filter)
            {
                continue;
            }
            match workflow.kind {
                ConnectionPluginWorkflowKind::Validator => {
                    diagnostics.push(validate_connection_workflow(workflow, message));
                }
                ConnectionPluginWorkflowKind::Manipulator => {
                    if let Err(error) = apply_connection_manipulator(workflow, message, direction) {
                        diagnostics.push(workflow_diagnostic(
                            workflow,
                            PluginDiagnosticSeverity::Error,
                            error,
                        ));
                    }
                }
            }
        }
        diagnostics
    }

    fn run_detail_formatter(
        &self,
        hook: &ActiveHook,
        bytes: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<FormattedMessageDetail, PluginHookError> {
        let Some(config) = self.parse_hook_config(hook, false) else {
            return Err(PluginHookError::failed(
                "detail formatter config is invalid",
            ));
        };
        let call = PluginHookCall {
            plugin_id: hook.plugin_id.clone(),
            hook: hook.hook,
            target: hook.target.clone(),
            config,
            input: PluginHookInput::DetailBytes {
                bytes,
                content_type,
            },
        };
        match self.plugin_hooks.execute(call)? {
            PluginHookOutput::DetailFormat(detail) => Ok(detail),
            output => Err(PluginHookError::failed(format!(
                "detail formatter returned incompatible output: {output:?}"
            ))),
        }
    }

    fn selected_detail_hook(&self, plugin_id: &str, kind: PluginHookKind) -> Option<ActiveHook> {
        self.active_hooks(kind)
            .into_iter()
            .find(|hook| hook.plugin_id == plugin_id)
    }

    fn active_topic_hooks(&self, kind: PluginHookKind, topic: &str) -> Vec<ActiveHook> {
        self.active_hooks(kind)
            .into_iter()
            .filter(|hook| topic_matches_filter(topic, &hook.target))
            .collect()
    }

    fn active_hooks(&self, kind: PluginHookKind) -> Vec<ActiveHook> {
        self.model
            .snapshot()
            .plugins
            .plugins
            .iter()
            .filter(|plugin| {
                plugin.enabled
                    && !matches!(
                        plugin.status,
                        PluginStatus::Disabled
                            | PluginStatus::NeedsConfig
                            | PluginStatus::CapabilityDenied
                            | PluginStatus::LoadError
                            | PluginStatus::UnsupportedLegacy
                    )
            })
            .flat_map(|plugin| {
                plugin
                    .hooks
                    .iter()
                    .filter(move |hook| {
                        hook.hook == kind
                            && hook.enabled
                            && matches!(hook.status, PluginHookStatus::Ready)
                    })
                    .map(|hook| ActiveHook {
                        plugin_id: plugin.id.clone(),
                        hook: hook.hook,
                        target: hook.target.clone(),
                        config_json: hook.config_json.clone(),
                    })
            })
            .collect()
    }

    fn parse_hook_config(&self, hook: &ActiveHook, publish_blocking: bool) -> Option<Value> {
        match serde_json::from_str::<Value>(&hook.config_json) {
            Ok(value) => Some(value),
            Err(error) => {
                let detail = format!("Hook config JSON is invalid: {error}");
                self.emit_hook_diagnostic(
                    hook,
                    PluginDiagnosticSeverity::Error,
                    "Plugin hook config is invalid.",
                    detail.clone(),
                    true,
                );
                if publish_blocking {
                    self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message: detail });
                }
                None
            }
        }
    }

    fn block_publish_for_output(
        &self,
        hook: &ActiveHook,
        output: PluginHookOutput,
    ) -> Option<PublishRequest> {
        let message = format!("Plugin hook returned incompatible output: {output:?}");
        self.emit_hook_diagnostic(
            hook,
            PluginDiagnosticSeverity::Error,
            "Publish hook failed.",
            message.clone(),
            true,
        );
        self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
        None
    }

    fn block_publish_for_error(
        &self,
        hook: &ActiveHook,
        error: PluginHookError,
    ) -> Option<PublishRequest> {
        let message = error.to_string();
        self.emit_hook_error(hook, "Publish hook failed.", error, true);
        self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
        None
    }

    fn emit_hook_error(
        &self,
        hook: &ActiveHook,
        message: &'static str,
        error: PluginHookError,
        mark_hook_failed: bool,
    ) {
        self.emit_hook_diagnostic(
            hook,
            PluginDiagnosticSeverity::Error,
            message,
            error.to_string(),
            mark_hook_failed,
        );
    }

    fn emit_hook_diagnostic(
        &self,
        hook: &ActiveHook,
        severity: PluginDiagnosticSeverity,
        message: impl Into<String>,
        detail: impl Into<String>,
        mark_hook_failed: bool,
    ) {
        self.emit_plugin_event(PluginWorkflowEvent::HookDiagnostic(
            PluginHookDiagnosticEvent {
                plugin_id: hook.plugin_id.clone(),
                hook: Some(hook.hook),
                severity,
                message: message.into(),
                detail: detail.into(),
                mark_hook_failed,
            },
        ));
    }

    fn emit_plugin_event(&self, event: PluginWorkflowEvent) {
        let _ = self.event_sender.emit(AppEvent::PluginWorkflow(event));
    }
}

fn plugin_window_messages(
    window: &PluginWindowRow,
    messages: &[crate::MessageRow],
) -> Vec<PluginWindowMessage> {
    let mut rows = Vec::new();
    let mut seen_topics = std::collections::BTreeSet::new();
    for message in messages {
        if let Some(prefix) = &window.message_filter_prefix {
            if !message.topic.starts_with(prefix) {
                continue;
            }
        }
        if window.latest_per_topic && !seen_topics.insert(message.topic.clone()) {
            continue;
        }
        rows.push(PluginWindowMessage {
            topic: message.topic.clone(),
            payload: message.payload.clone(),
            qos: message.qos,
            retained: message.retained,
            timestamp: message.timestamp.clone(),
        });
    }
    rows
}

fn plugin_host_action_to_mqtt(action: PluginHostAction) -> Result<MqttCommand, String> {
    match action {
        PluginHostAction::Subscribe {
            connection_id,
            topic_filter,
            qos,
        } => Ok(MqttCommand::Subscribe {
            connection_id,
            subscription: Subscription::new(topic_filter.as_str(), mqtt_qos(qos))
                .map_err(|source| source.to_report().message)?,
        }),
        PluginHostAction::Unsubscribe {
            connection_id,
            topic_filter,
        } => Ok(MqttCommand::Unsubscribe {
            connection_id,
            request: UnsubscribeRequest::new(topic_filter.as_str())
                .map_err(|source| source.to_report().message)?,
        }),
    }
}

fn workflow_direction_matches(
    configured: ConnectionPluginDirection,
    actual: ConnectionPluginDirection,
) -> bool {
    configured == ConnectionPluginDirection::Both || configured == actual
}

fn validate_connection_workflow(
    workflow: &ConnectionPluginWorkflow,
    message: &crate::PluginMessage,
) -> MessageDiagnosticRow {
    match workflow.plugin_id.as_str() {
        "org.correomqtt.plugins.contains-string-validator" => {
            let payload = String::from_utf8_lossy(&message.payload);
            let rules = workflow
                .config
                .get("rules")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            let valid = rules.is_empty()
                || rules.iter().any(|rule| {
                    let Some(needle) = rule.get("text").and_then(serde_json::Value::as_str) else {
                        return false;
                    };
                    if rule
                        .get("regex")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                    {
                        regex::Regex::new(needle)
                            .map(|regex| regex.is_match(&payload))
                            .unwrap_or(false)
                    } else {
                        payload.contains(needle)
                    }
                });
            if valid {
                validator_workflow_diagnostic(
                    workflow,
                    PluginDiagnosticSeverity::Info,
                    "Validation passed",
                )
            } else {
                validator_workflow_diagnostic(
                    workflow,
                    PluginDiagnosticSeverity::Error,
                    "Payload did not match configured string rules",
                )
            }
        }
        "org.correomqtt.plugins.xml-xsd-validator" => {
            let payload = String::from_utf8_lossy(&message.payload);
            let xsd_path = workflow
                .config
                .get("xsd_path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if payload.trim_start().starts_with('<') && !xsd_path.trim().is_empty() {
                validator_workflow_diagnostic(
                    workflow,
                    PluginDiagnosticSeverity::Info,
                    "Validation passed",
                )
            } else {
                validator_workflow_diagnostic(
                    workflow,
                    PluginDiagnosticSeverity::Error,
                    "XML/XSD validation requires XML payload and configured XSD file",
                )
            }
        }
        _ => validator_workflow_diagnostic(
            workflow,
            PluginDiagnosticSeverity::Info,
            "Validation passed",
        ),
    }
}

fn apply_connection_manipulator(
    workflow: &ConnectionPluginWorkflow,
    message: &mut crate::PluginMessage,
    direction: ConnectionPluginDirection,
) -> Result<(), String> {
    match workflow.plugin_id.as_str() {
        "org.correomqtt.plugins.base64" => {
            if direction == ConnectionPluginDirection::Outgoing {
                message.payload = base64::engine::general_purpose::STANDARD
                    .encode(&message.payload)
                    .into_bytes();
            } else {
                message.payload = base64::engine::general_purpose::STANDARD
                    .decode(&message.payload)
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        }
        "org.correomqtt.plugins.zip-manipulator" => {
            if direction == ConnectionPluginDirection::Outgoing {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(&message.payload)
                    .map_err(|error| error.to_string())?;
                message.payload = encoder.finish().map_err(|error| error.to_string())?;
            } else {
                let mut decoder = GzDecoder::new(message.payload.as_slice());
                let mut decoded = Vec::new();
                decoder
                    .read_to_end(&mut decoded)
                    .map_err(|error| error.to_string())?;
                message.payload = decoded;
            }
            Ok(())
        }
        "org.correomqtt.plugins.save-manipulator" => {
            let folder = workflow
                .config
                .get("folder")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim();
            if folder.is_empty() {
                return Err("Save folder is not configured".to_owned());
            }
            let folder = std::path::Path::new(folder);
            std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
            let path = folder.join(format!(
                "{}-{}.payload",
                match direction {
                    ConnectionPluginDirection::Incoming => "incoming",
                    ConnectionPluginDirection::Outgoing => "outgoing",
                    ConnectionPluginDirection::Both => "message",
                },
                sanitize_file_name(&message.topic)
            ));
            std::fs::write(path, &message.payload).map_err(|error| error.to_string())
        }
        _ => Ok(()),
    }
}

fn sanitize_file_name(value: &str) -> String {
    let name = value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect::<String>();
    let name = name.trim_matches('-');
    if name.is_empty() {
        "message".to_owned()
    } else {
        name.to_owned()
    }
}

fn workflow_diagnostic(
    workflow: &ConnectionPluginWorkflow,
    severity: PluginDiagnosticSeverity,
    message: impl Into<String>,
) -> MessageDiagnosticRow {
    MessageDiagnosticRow {
        severity,
        hook: None,
        plugin_id: Some(workflow.plugin_id.clone()),
        message: format!("{}: {}", workflow.plugin_name, message.into()),
    }
}

fn validator_workflow_diagnostic(
    workflow: &ConnectionPluginWorkflow,
    severity: PluginDiagnosticSeverity,
    message: impl Into<String>,
) -> MessageDiagnosticRow {
    let mut diagnostic = workflow_diagnostic(workflow, severity, message);
    diagnostic.hook = Some(PluginHookKind::Validator);
    diagnostic
}
