impl AppRuntime {
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
        PluginHostAction::SavePayload(_) => {
            Err("save payload actions must be handled by the host save queue".to_owned())
        }
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
