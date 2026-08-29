#[derive(Debug)]
pub struct InstalledPluginExecutor {
    registry: Mutex<PluginRegistry>,
    active_calls: Mutex<Vec<(u64, PluginCancellationToken)>>,
    next_call_id: AtomicU64,
    config_root: PathBuf,
}

struct ActivePluginCall<'a> {
    id: u64,
    calls: &'a Mutex<Vec<(u64, PluginCancellationToken)>>,
}

impl Drop for ActivePluginCall<'_> {
    fn drop(&mut self) {
        if let Ok(mut calls) = self.calls.lock() {
            calls.retain(|(id, _)| *id != self.id);
        }
    }
}

impl InstalledPluginExecutor {
    pub fn load(config_root: PathBuf, package_dirs: &[PathBuf]) -> Result<Self, String> {
        log_plugin_info(format!(
            "runtime: preparing lazy plugin registry for {} installed package directory/directories",
            package_dirs.len()
        ));
        let version =
            semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| error.to_string())?;
        let registry = PluginRegistry::new(version).map_err(|error| error.to_string())?;
        log_plugin_info(
            "runtime: plugin registry ready; WASM packages will compile on first hook use"
                .to_owned(),
        );
        Ok(Self {
            registry: Mutex::new(registry),
            active_calls: Mutex::new(Vec::new()),
            next_call_id: AtomicU64::new(0),
            config_root,
        })
    }
}

impl PluginHookExecutor for InstalledPluginExecutor {
    fn execute(&self, call: PluginHookCall) -> Result<PluginHookOutput, PluginHookError> {
        self.execute_with_host_actions(call)
            .map(|execution| execution.output)
    }

    fn execute_with_host_actions(
        &self,
        call: PluginHookCall,
    ) -> Result<PluginHookExecution, PluginHookError> {
        let (plugin, cancellation) = {
            let mut registry = self
                .registry
                .lock()
                .map_err(|_| PluginHookError::failed("plugin registry lock is poisoned"))?;
            if registry.get(&call.plugin_id).is_none() {
                let package_dir = plugin_install_dir(&self.config_root, &call.plugin_id);
                log_plugin_info(format!(
                    "runtime: lazy-loading plugin {} from {}",
                    call.plugin_id,
                    package_dir.display()
                ));
                let package = PluginPackage::load(package_dir)
                    .map_err(|error| PluginHookError::failed(error.to_string()))?;
                registry
                    .register_package(package)
                    .map_err(|error| PluginHookError::failed(error.to_string()))?;
            }
            let plugin = registry.get(&call.plugin_id).cloned().ok_or_else(|| {
                PluginHookError::failed(format!("plugin {} is not loaded", call.plugin_id))
            })?;
            (plugin, registry.cancellation_token())
        };
        let call_id = self.next_call_id.fetch_add(1, Ordering::Relaxed);
        self.active_calls
            .lock()
            .map_err(|_| PluginHookError::failed("active plugin call lock is poisoned"))?
            .push((call_id, cancellation.clone()));
        let _active_call = ActivePluginCall {
            id: call_id,
            calls: &self.active_calls,
        };
        let host_capabilities = plugin.manifest().capabilities.host.clone();
        let abi = plugin.abi();
        let v1_transport_input = (abi == PluginAbi::V1)
            .then(|| match &call.input {
                PluginHookInput::TransportMessage(input) => Some(input.clone()),
                _ => None,
            })
            .flatten();
        let invocation = hook_invocation(call, abi)?;
        let dispatch = plugin.dispatch_with_cancel(invocation, &cancellation);
        dispatch
            .map_err(|error| match error {
                correo_plugins::HookDispatchError::Cancelled { .. } => {
                    PluginHookError::cancelled(error.to_string())
                }
                _ => PluginHookError::failed(error.to_string()),
            })
            .and_then(|output| hook_execution(output, &host_capabilities, v1_transport_input))
    }

    fn cancel(&self) {
        if let Ok(active_calls) = self.active_calls.lock() {
            for (_, cancellation) in active_calls.iter() {
                cancellation.cancel();
            }
        }
    }

    fn highlight_payload(
        &self,
        text: &str,
        active_plugin_ids: &[String],
    ) -> Option<Vec<PayloadSyntaxSpan>> {
        let xml_active = active_plugin_ids
            .iter()
            .any(|plugin_id| plugin_id == correo_plugins::XML_FORMAT_ID);
        let json_active = active_plugin_ids
            .iter()
            .any(|plugin_id| plugin_id == correo_plugins::JSON_FORMAT_ID);

        if xml_active {
            if let Some(spans) = correo_plugins::highlight_xml(text) {
                return Some(spans.into_iter().map(payload_syntax_span).collect());
            }
        }
        if json_active {
            correo_plugins::highlight_json(text)
                .map(|spans| spans.into_iter().map(payload_syntax_span).collect())
        } else {
            None
        }
    }

    fn connection_action(
        &self,
        request: PluginConnectionActionRequest,
    ) -> Result<PluginConnectionActionResponse, PluginHookError> {
        if request.plugin_id != correo_plugins_systopic::PLUGIN_ID {
            return Err(PluginHookError::failed(format!(
                "plugin {} does not provide connection actions",
                request.plugin_id
            )));
        }
        let response = correo_plugins_systopic::connection_action_clicked(
            &request.action_id,
            &request.connection_name,
        )
        .ok_or_else(|| PluginHookError::failed("unknown plugin connection action"))?;
        Ok(PluginConnectionActionResponse {
            host_actions: response
                .host_actions
                .into_iter()
                .map(|action| sys_topic_host_action(request.connection_id, action))
                .collect(),
            open_window: Some(PluginOpenWindow {
                plugin_id: request.plugin_id,
                action_id: request.action_id,
                connection_id: request.connection_id,
                title: response.open_window.title,
                message_filter_prefix: Some(response.open_window.message_filter_prefix),
                latest_per_topic: response.open_window.latest_per_topic,
            }),
        })
    }

    fn close_window(
        &self,
        request: PluginWindowCloseRequest,
    ) -> Result<PluginHostActionResponse, PluginHookError> {
        if request.plugin_id != correo_plugins_systopic::PLUGIN_ID {
            return Ok(PluginHostActionResponse::default());
        }
        let response = correo_plugins_systopic::connection_window_closed(&request.action_id)
            .ok_or_else(|| PluginHookError::failed("unknown plugin window"))?;
        Ok(PluginHostActionResponse {
            host_actions: response
                .host_actions
                .into_iter()
                .map(|action| sys_topic_host_action(request.connection_id, action))
                .collect(),
        })
    }

    fn render_window(
        &self,
        request: PluginWindowRenderRequest,
    ) -> Result<PluginWindowRenderResponse, PluginHookError> {
        if request.plugin_id != correo_plugins_systopic::PLUGIN_ID {
            return Err(PluginHookError::failed(format!(
                "plugin {} does not provide window rendering",
                request.plugin_id
            )));
        }
        let messages = request
            .messages
            .into_iter()
            .map(sys_topic_message)
            .collect::<Vec<_>>();
        let nodes =
            correo_plugins_systopic::render_window(&request.action_id, &request.broker, &messages)
                .ok_or_else(|| PluginHookError::failed("unknown plugin window"))?
                .into_iter()
                .map(sys_topic_ui_node)
                .collect();
        Ok(PluginWindowRenderResponse { nodes })
    }
}

fn sys_topic_host_action(
    connection_id: correo_mqtt::ConnectionId,
    action: correo_plugins_systopic::SysTopicHostAction,
) -> PluginHostAction {
    match action {
        correo_plugins_systopic::SysTopicHostAction::Subscribe { topic_filter } => {
            PluginHostAction::Subscribe {
                connection_id,
                topic_filter,
                qos: QosLevel::Zero,
            }
        }
        correo_plugins_systopic::SysTopicHostAction::Unsubscribe { topic_filter } => {
            PluginHostAction::Unsubscribe {
                connection_id,
                topic_filter,
            }
        }
    }
}

fn sys_topic_message(message: PluginWindowMessage) -> correo_plugins_systopic::SysTopicMessage {
    correo_plugins_systopic::SysTopicMessage {
        topic: message.topic,
        payload: message.payload,
        qos: message.qos.label().to_owned(),
        retained: message.retained,
        timestamp: message.timestamp,
    }
}

fn sys_topic_ui_node(node: correo_plugins_systopic::SysTopicUiNode) -> PluginUiNode {
    match node {
        correo_plugins_systopic::SysTopicUiNode::Heading { text } => PluginUiNode::Heading { text },
        correo_plugins_systopic::SysTopicUiNode::Label { text } => PluginUiNode::Label { text },
        correo_plugins_systopic::SysTopicUiNode::Separator => PluginUiNode::Separator,
        correo_plugins_systopic::SysTopicUiNode::Table { columns, rows } => {
            PluginUiNode::Table { columns, rows }
        }
        correo_plugins_systopic::SysTopicUiNode::MetricList(list) => {
            PluginUiNode::MetricList(PluginMetricListNode {
                broker: list.broker,
                latest_update: list.latest_update,
                copy_text: list.copy_text,
                rows: list
                    .rows
                    .into_iter()
                    .map(|row| PluginMetricRow {
                        name: row.name,
                        description: row.description,
                        value: row.value,
                    })
                    .collect(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginFileInstaller {
    config_root: PathBuf,
    executable_dir: PathBuf,
}

impl PluginFileInstaller {
    pub fn new(config_root: PathBuf) -> Self {
        let executable_dir = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            config_root,
            executable_dir,
        }
    }
}

impl PluginInstaller for PluginFileInstaller {
    fn install(&self, plugin: &PluginMarketplaceRow) -> Result<String, String> {
        log_plugin_info(format!("marketplace: install requested for {}", plugin.id));
        let repository = LoadedRepository {
            id: "marketplace".to_owned(),
            json: String::new(),
            base_dir: Some(self.executable_dir.clone()),
            rows: Vec::new(),
        };
        install_marketplace_plugin(&self.config_root, &repository, plugin)?;
        Ok(plugin_install_dir(&self.config_root, &plugin.id)
            .to_string_lossy()
            .into_owned())
    }

    fn uninstall(&self, plugin_id: &str) -> Result<(), String> {
        let path = plugin_install_dir(&self.config_root, plugin_id);
        if path.exists() {
            log_plugin_info(format!(
                "marketplace: uninstalling {plugin_id} from {}",
                path.display()
            ));
            fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        } else {
            log_plugin_info(format!(
                "marketplace: uninstall requested for {plugin_id}, but no installed directory exists"
            ));
        }
        Ok(())
    }
}

fn hook_invocation(
    call: PluginHookCall,
    abi: PluginAbi,
) -> Result<HookInvocation, PluginHookError> {
    match (call.hook, call.input) {
        (PluginHookKind::OutgoingTransform, PluginHookInput::TransportMessage(message))
            if abi == PluginAbi::V1 =>
        {
            Ok(HookInvocation::OutgoingMessageTransform(
                OutgoingMessageTransformRequest {
                    abi_version: correo_plugins::ABI_VERSION,
                    context: HookContextDto::default(),
                    config: call.config,
                    message: message_dto(legacy_message_from_transport(message)?),
                },
            ))
        }
        (PluginHookKind::IncomingTransform, PluginHookInput::TransportMessage(message))
            if abi == PluginAbi::V1 =>
        {
            Ok(HookInvocation::IncomingMessageTransform(
                IncomingMessageTransformRequest {
                    abi_version: correo_plugins::ABI_VERSION,
                    context: HookContextDto::default(),
                    config: call.config,
                    message: message_dto(legacy_message_from_transport(message)?),
                },
            ))
        }
        (PluginHookKind::Validator, PluginHookInput::TransportMessage(message))
            if abi == PluginAbi::V1 =>
        {
            Ok(HookInvocation::MessageValidator(MessageValidatorRequest {
                abi_version: correo_plugins::ABI_VERSION,
                context: HookContextDto::default(),
                config: call.config,
                message: message_dto(legacy_message_from_transport(message)?),
            }))
        }
        (PluginHookKind::OutgoingTransform, PluginHookInput::Message(message))
            if abi == PluginAbi::V1 =>
        {
            Ok(HookInvocation::OutgoingMessageTransform(
                OutgoingMessageTransformRequest {
                    abi_version: correo_plugins::ABI_VERSION,
                    context: HookContextDto::default(),
                    config: call.config,
                    message: message_dto(message),
                },
            ))
        }
        (PluginHookKind::IncomingTransform, PluginHookInput::Message(message))
            if abi == PluginAbi::V1 =>
        {
            Ok(HookInvocation::IncomingMessageTransform(
                IncomingMessageTransformRequest {
                    abi_version: correo_plugins::ABI_VERSION,
                    context: HookContextDto::default(),
                    config: call.config,
                    message: message_dto(message),
                },
            ))
        }
        (PluginHookKind::Validator, PluginHookInput::Message(message)) if abi == PluginAbi::V1 => {
            Ok(HookInvocation::MessageValidator(MessageValidatorRequest {
                abi_version: correo_plugins::ABI_VERSION,
                context: HookContextDto::default(),
                config: call.config,
                message: message_dto(message),
            }))
        }
        (
            PluginHookKind::DetailTransform,
            PluginHookInput::DetailBytes {
                bytes,
                content_type,
            },
        ) if abi == PluginAbi::V1 => Ok(HookInvocation::DetailByteTransform(
            DetailByteTransformRequest {
                abi_version: correo_plugins::ABI_VERSION,
                context: HookContextDto::default(),
                config: call.config,
                bytes,
                content_type,
            },
        )),
        (
            PluginHookKind::DetailFormatter,
            PluginHookInput::DetailBytes {
                bytes,
                content_type,
            },
        ) if abi == PluginAbi::V1 => Ok(HookInvocation::DetailFormatter(DetailFormatterRequest {
            abi_version: correo_plugins::ABI_VERSION,
            context: HookContextDto::default(),
            config: call.config,
            bytes,
            content_type,
        })),
        (PluginHookKind::OutgoingTransform, PluginHookInput::TransportMessage(message))
            if abi == PluginAbi::V2 =>
        {
            Ok(HookInvocation::OutgoingTransportMessageTransform(
                OutgoingTransportMessageTransformRequest {
                    abi_version: correo_plugins::ABI_VERSION_V2,
                    context: HookContextDto::default(),
                    config: call.config,
                    input: transport_input_dto(message)?,
                },
            ))
        }
        (PluginHookKind::IncomingTransform, PluginHookInput::TransportMessage(message))
            if abi == PluginAbi::V2 =>
        {
            Ok(HookInvocation::IncomingTransportMessageTransform(
                IncomingTransportMessageTransformRequest {
                    abi_version: correo_plugins::ABI_VERSION_V2,
                    context: HookContextDto::default(),
                    config: call.config,
                    input: transport_input_dto(message)?,
                },
            ))
        }
        (PluginHookKind::Validator, PluginHookInput::TransportMessage(message))
            if abi == PluginAbi::V2 =>
        {
            Ok(HookInvocation::TransportMessageValidator(
                TransportMessageValidatorRequest {
                    abi_version: correo_plugins::ABI_VERSION_V2,
                    context: HookContextDto::default(),
                    config: call.config,
                    input: transport_input_dto(message)?,
                },
            ))
        }
        _ => Err(PluginHookError::failed(
            "plugin hook input did not match hook kind",
        )),
    }
}
