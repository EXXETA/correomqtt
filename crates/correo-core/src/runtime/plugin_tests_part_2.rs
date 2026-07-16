#[tokio::test]
async fn incoming_hooks_are_ordered_nonblocking_and_cancellable() {
    let mut snapshot = sample_snapshot(ThemeMode::System);
    enable_hook(
        &mut snapshot,
        "org.correomqtt.plugins.base64",
        PluginHookKind::IncomingTransform,
        "bridge/#",
    );
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = AppRuntime::with_snapshot(snapshot);
    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::IncomingSlow(Duration::from_millis(100)),
        calls.clone(),
    ));
    runtime.attach_mqtt_service(
        crate::MqttService::spawn(FakeFactory::new(Arc::default(), None)).unwrap(),
    );
    let connection_id = runtime.snapshot().connections[2].id;
    runtime
        .command_sender()
        .send(AppCommand::Connect(connection_id))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    for topic in ["bridge/first", "bridge/second"] {
        runtime
            .command_sender()
            .send(AppCommand::Mqtt(Box::new(MqttCommand::Publish {
                connection_id,
                request: PublishRequest::new(topic, b"payload".to_vec(), Qos::AtMostOnce, false)
                    .unwrap(),
                diagnostics: Vec::new(),
            })))
            .unwrap();
    }
    let started = Instant::now();
    runtime.pump();
    assert!(
        started.elapsed() < Duration::from_millis(50),
        "pump must not wait for a slow incoming hook"
    );
    pump_until(&mut runtime, |runtime| {
        ["bridge/first", "bridge/second"].iter().all(|topic| {
            runtime
                .snapshot()
                .workbench
                .messages
                .iter()
                .any(|message| message.topic == *topic)
        })
    })
    .await;
    let topics = calls
        .lock()
        .unwrap()
        .iter()
        .filter_map(|call| match &call.input {
            PluginHookInput::TransportMessage(message)
                if call.hook == PluginHookKind::IncomingTransform =>
            {
                Some(message.message.address.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(topics[..2], ["bridge/first", "bridge/second"]);

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Publish {
            connection_id,
            request: PublishRequest::new(
                "bridge/cancelled",
                b"payload".to_vec(),
                Qos::AtMostOnce,
                false,
            )
            .unwrap(),
            diagnostics: Vec::new(),
        })))
        .unwrap();
    pump_until(&mut runtime, |_| {
        calls.lock().unwrap().iter().any(|call| {
            matches!(
                &call.input,
                PluginHookInput::TransportMessage(message)
                    if message.message.address == "bridge/cancelled"
            )
        })
    })
    .await;
    runtime.command_sender().send(AppCommand::Shutdown).unwrap();
    runtime.pump();
    pump_until(&mut runtime, |runtime| {
        runtime.snapshot().diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("Incoming plugin processing cancelled")
        })
    })
    .await;
    assert!(!runtime
        .snapshot()
        .workbench
        .messages
        .iter()
        .any(|message| message.topic == "bridge/cancelled"));
    assert!(runtime.snapshot().diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Incoming plugin processing cancelled")
    }));
}
#[derive(Debug)]
struct MockHooks {
    behavior: MockBehavior,
    calls: Arc<Mutex<Vec<PluginHookCall>>>,
    cancelled: AtomicBool,
}

impl MockHooks {
    fn new(behavior: MockBehavior, calls: Arc<Mutex<Vec<PluginHookCall>>>) -> Self {
        Self {
            behavior,
            calls,
            cancelled: AtomicBool::new(false),
        }
    }
}

impl PluginHookExecutor for MockHooks {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn execute(&self, call: PluginHookCall) -> Result<PluginHookOutput, PluginHookError> {
        self.calls.lock().unwrap().push(call.clone());
        match (&self.behavior, call.hook, call.input) {
            (
                MockBehavior::OutgoingReplace(payload),
                PluginHookKind::OutgoingTransform,
                PluginHookInput::TransportMessage(mut message),
            ) => {
                message.message.body = payload.clone();
                Ok(PluginHookOutput::TransportMessageTransform(
                    crate::TransportMessageTransform::Replace(message),
                ))
            }
            (
                MockBehavior::ValidatorBlock(message),
                PluginHookKind::Validator,
                PluginHookInput::TransportMessage(_),
            ) => Ok(PluginHookOutput::Validation(PluginValidation::Block {
                message: message.clone(),
            })),
            (
                MockBehavior::ValidatorWarning(message),
                PluginHookKind::Validator,
                PluginHookInput::TransportMessage(_),
            ) => Ok(PluginHookOutput::Validation(PluginValidation::Warning {
                message: message.clone(),
            })),
            (
                MockBehavior::IncomingError(message),
                PluginHookKind::IncomingTransform,
                PluginHookInput::TransportMessage(_),
            ) => Err(PluginHookError::failed(message.clone())),
            (
                MockBehavior::IncomingSlow(delay),
                PluginHookKind::IncomingTransform,
                PluginHookInput::TransportMessage(message),
            ) => {
                let delay = if message.message.address == "bridge/cancelled" {
                    Duration::from_secs(5)
                } else {
                    *delay
                };
                let deadline = Instant::now() + delay;
                while Instant::now() < deadline {
                    if self.cancelled.load(Ordering::Acquire) {
                        return Err(PluginHookError::cancelled(
                            "incoming hook execution cancelled",
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
                Ok(PluginHookOutput::TransportMessageTransform(
                    crate::TransportMessageTransform::Replace(message),
                ))
            }
            (
                MockBehavior::DetailFormat(text),
                PluginHookKind::DetailFormatter,
                PluginHookInput::DetailBytes { .. },
            ) => Ok(PluginHookOutput::DetailFormat(FormattedMessageDetail {
                format: MessageDetailFormat::Json,
                text: text.clone(),
                content_type: Some("application/json".to_owned()),
                diagnostics: Vec::new(),
            })),
            (
                MockBehavior::DetailCancel(message),
                PluginHookKind::DetailFormatter,
                PluginHookInput::DetailBytes { .. },
            ) => Err(PluginHookError::cancelled(message.clone())),
            (
                MockBehavior::DetailSave,
                PluginHookKind::DetailTransform,
                PluginHookInput::DetailBytes { .. },
            ) => Ok(PluginHookOutput::DetailBytes(crate::DetailBytesOutput {
                bytes: b"transformed".to_vec(),
                content_type: Some("application/octet-stream".to_owned()),
            })),
            (_, PluginHookKind::Validator, _) => {
                Ok(PluginHookOutput::Validation(PluginValidation::Valid))
            }
            (
                _,
                PluginHookKind::DetailTransform,
                PluginHookInput::DetailBytes {
                    bytes,
                    content_type,
                },
            ) => Ok(PluginHookOutput::DetailBytes(crate::DetailBytesOutput {
                bytes,
                content_type,
            })),
            (_, _, PluginHookInput::Message(_)) => Err(PluginHookError::failed(
                "legacy message hook input is unsupported",
            )),
            (_, _, PluginHookInput::TransportMessage(message)) => {
                Ok(PluginHookOutput::TransportMessageTransform(
                    crate::TransportMessageTransform::Replace(message),
                ))
            }
            (
                _,
                _,
                PluginHookInput::DetailBytes {
                    bytes,
                    content_type,
                },
            ) => Ok(PluginHookOutput::DetailFormat(FormattedMessageDetail {
                format: MessageDetailFormat::PlainText,
                text: String::from_utf8_lossy(&bytes).into_owned(),
                content_type,
                diagnostics: Vec::new(),
            })),
        }
    }

    fn execute_with_host_actions(
        &self,
        call: PluginHookCall,
    ) -> Result<PluginHookExecution, PluginHookError> {
        let host_actions = match (&self.behavior, &call.hook) {
            (MockBehavior::DetailSave, PluginHookKind::DetailTransform) => {
                vec![PluginHostAction::SavePayload(PluginSavePayload {
                    suggested_file_name: "transformed.bin".to_owned(),
                    bytes: b"saved".to_vec(),
                    content_type: Some("application/octet-stream".to_owned()),
                })]
            }
            _ => Vec::new(),
        };
        self.execute(call).map(|output| PluginHookExecution {
            output,
            host_actions,
        })
    }
}

#[derive(Debug)]
enum MockBehavior {
    OutgoingReplace(Vec<u8>),
    ValidatorBlock(String),
    ValidatorWarning(String),
    IncomingError(String),
    IncomingSlow(Duration),
    DetailFormat(String),
    DetailCancel(String),
    DetailSave,
}

fn enable_hook(
    snapshot: &mut crate::AppSnapshot,
    plugin_id: &str,
    kind: PluginHookKind,
    target: &str,
) {
    let plugin = snapshot
        .plugins
        .plugins
        .iter_mut()
        .find(|plugin| plugin.id == plugin_id)
        .unwrap();
    plugin.enabled = true;
    plugin.status = PluginStatus::Active;
    let hook = plugin
        .hooks
        .iter_mut()
        .find(|hook| hook.hook == kind)
        .unwrap();
    hook.enabled = true;
    hook.status = PluginHookStatus::Ready;
    hook.target = target.to_owned();
    hook.config_json = "{}".to_owned();
}
