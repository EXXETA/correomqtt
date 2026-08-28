impl AppRuntime {
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

    pub(super) fn refresh_message_detail(&mut self, allow_host_actions: bool) {
        let snapshot = self.model.snapshot().clone();
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
                    Ok((output, host_actions)) => {
                        if allow_host_actions {
                            self.forward_plugin_host_actions(host_actions, true);
                        }
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
        let retained_when_absent = request.retain;
        let mut transport = plugin_transport_message_from_publish(&request, message);
        let transport_capabilities = transport.capabilities.clone();

        for hook in self.active_topic_hooks(PluginHookKind::Validator, &topic) {
            let config = self.parse_hook_config(&hook, true)?;
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

        let message = match plugin_message_from_transport(
            transport,
            retained_when_absent,
            &transport_capabilities,
        ) {
            Ok(message) => message,
            Err(message) => {
                self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
                return None;
            }
        };
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
    ) -> Result<(DetailBytesOutput, Vec<PluginHostAction>), PluginHookError> {
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
        let execution = self.plugin_hooks.execute_with_host_actions(call)?;
        match execution.output {
            PluginHookOutput::DetailBytes(output) => Ok((output, execution.host_actions)),
            output => Err(PluginHookError::failed(format!(
                "detail transform returned incompatible output: {output:?}"
            ))),
        }
    }

}
