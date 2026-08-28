use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

use correo_mqtt::{IncomingMessage, PublishRequest, Qos, TopicName};

use crate::mqtt::test_support::{connection_options, connection_state, pump_until, FakeFactory};
use crate::{
    sample_snapshot, AppCommand, AppRuntime, ConnectionState, FormattedMessageDetail,
    MessageDetailFormat, MqttCommand, PluginHookCall, PluginHookError, PluginHookExecution,
    PluginHookExecutor, PluginHookInput, PluginHookKind, PluginHookOutput, PluginHookStatus,
    PluginHostAction, PluginSavePayload, PluginStatus, PluginValidation, ThemeMode,
};

#[tokio::test]
async fn publish_hooks_transform_wire_payload_and_keep_draft() {
    let mut snapshot = sample_snapshot(ThemeMode::System);
    enable_hook(
        &mut snapshot,
        "org.correomqtt.plugins.base64",
        PluginHookKind::OutgoingTransform,
        "bridge/#",
    );
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = AppRuntime::with_snapshot(snapshot);
    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::OutgoingReplace(b"wire".to_vec()),
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

    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishTopic(
            "bridge/device/set".to_owned(),
        ))
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishPayload("draft".to_owned()))
        .unwrap();
    runtime.command_sender().send(AppCommand::Publish).unwrap();

    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .messages
            .first()
            .is_some_and(|message| message.payload_preview == "wire")
    })
    .await;

    assert_eq!(runtime.snapshot().workbench.publish.payload, "draft");
    assert!(runtime
        .snapshot()
        .workbench
        .publish
        .history
        .iter()
        .any(|row| row.topic == "bridge/device/set" && row.byte_size == 4));
    assert!(calls
        .lock()
        .unwrap()
        .iter()
        .any(|call| call.hook == PluginHookKind::OutgoingTransform));
}

#[test]
fn validator_blocks_publish_with_redacted_feedback_and_preserved_draft() {
    let mut snapshot = sample_snapshot(ThemeMode::System);
    enable_hook(
        &mut snapshot,
        "user.advanced-validator",
        PluginHookKind::Validator,
        "blocked/#",
    );
    let mut runtime = AppRuntime::with_snapshot(snapshot);
    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::ValidatorBlock("password=synthetic-secret rejected".to_owned()),
        Arc::default(),
    ));

    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishTopic("blocked/device".to_owned()))
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishPayload(
            "draft-secret-free".to_owned(),
        ))
        .unwrap();
    runtime.command_sender().send(AppCommand::Publish).unwrap();
    runtime.pump();
    runtime.pump();

    let feedback = runtime
        .snapshot()
        .workbench
        .publish
        .feedback
        .as_ref()
        .unwrap();
    assert!(feedback.message.contains("[REDACTED]"));
    assert!(!feedback.message.contains("synthetic-secret"));
    assert_eq!(
        runtime.snapshot().workbench.publish.payload,
        "draft-secret-free"
    );
}

#[test]
fn validator_warning_allows_publish_and_reports_feedback() {
    let mut snapshot = sample_snapshot(ThemeMode::System);
    enable_hook(
        &mut snapshot,
        "user.advanced-validator",
        PluginHookKind::Validator,
        "telemetry/+/set",
    );
    let mut runtime = AppRuntime::with_snapshot(snapshot);
    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::ValidatorWarning("validator warning only".to_owned()),
        Arc::default(),
    ));

    let commands = runtime
        .mqtt_commands_for_app_command_with_plugins(&AppCommand::Publish)
        .unwrap();
    assert!(commands
        .iter()
        .any(|command| matches!(command, MqttCommand::Publish { .. })));

    runtime.pump();
    let feedback = runtime
        .snapshot()
        .workbench
        .publish
        .feedback
        .as_ref()
        .unwrap();
    assert_eq!(feedback.message, "validator warning only");
}

#[tokio::test]
async fn incoming_transform_error_keeps_payload_and_records_diagnostic() {
    let mut snapshot = sample_snapshot(ThemeMode::System);
    enable_hook(
        &mut snapshot,
        "org.correomqtt.plugins.base64",
        PluginHookKind::IncomingTransform,
        "bridge/#",
    );
    let mut runtime = AppRuntime::with_snapshot(snapshot);
    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::IncomingError("secret=incoming-secret failed".to_owned()),
        Arc::default(),
    ));
    runtime.attach_mqtt_service(
        crate::MqttService::spawn(FakeFactory::new(Arc::default(), None)).unwrap(),
    );
    let connection_id = runtime.snapshot().connections[2].id;
    runtime
        .command_sender()
        .send(AppCommand::SelectConnection(connection_id))
        .unwrap();
    runtime.pump();

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Connect {
            options: connection_options(connection_id),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Publish {
            connection_id,
            request: PublishRequest::new("bridge/raw", b"online".to_vec(), Qos::AtMostOnce, false)
                .unwrap(),
            diagnostics: Vec::new(),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .messages
            .first()
            .is_some_and(|message| !message.diagnostics.is_empty())
    })
    .await;

    let message = runtime.snapshot().workbench.messages.first().unwrap();
    assert_eq!(message.payload_preview, "online");
    let diagnostics = format!("{:?}", message.diagnostics);
    assert!(diagnostics.contains("[REDACTED]"));
    assert!(!diagnostics.contains("incoming-secret"));
}

#[tokio::test]
async fn incoming_validator_result_is_recorded_on_message() {
    let mut snapshot = sample_snapshot(ThemeMode::System);
    enable_hook(
        &mut snapshot,
        "user.advanced-validator",
        PluginHookKind::Validator,
        "bridge/#",
    );
    let mut runtime = AppRuntime::with_snapshot(snapshot);
    // A blocking validator now cancels the publish before the fake session can
    // loop it back, so a warning validator exercises the incoming recording.
    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::ValidatorWarning("payload missing required text".to_owned()),
        Arc::default(),
    ));
    runtime.attach_mqtt_service(
        crate::MqttService::spawn(FakeFactory::new(Arc::default(), None)).unwrap(),
    );
    let connection_id = runtime.snapshot().connections[2].id;
    runtime
        .command_sender()
        .send(AppCommand::SelectConnection(connection_id))
        .unwrap();
    runtime.pump();

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Connect {
            options: connection_options(connection_id),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Publish {
            connection_id,
            request: PublishRequest::new("bridge/raw", b"online".to_vec(), Qos::AtMostOnce, false)
                .unwrap(),
            diagnostics: Vec::new(),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .messages
            .first()
            .is_some_and(|message| {
                message.diagnostics.iter().any(|diagnostic| {
                    diagnostic.message == "payload missing required text"
                        && diagnostic.severity == crate::PluginDiagnosticSeverity::Warning
                })
            })
    })
    .await;
}

#[test]
fn detail_formatter_selection_renders_and_cancellation_falls_back() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = AppRuntime::new();
    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::DetailFormat("{\n  \"formatted\": true\n}".to_owned()),
        calls.clone(),
    ));
    runtime
        .command_sender()
        .send(AppCommand::SelectDetailFormatter(Some(
            "org.correomqtt.plugins.json-format".to_owned(),
        )))
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::RefreshMessageDetail)
        .unwrap();
    runtime.pump();
    runtime.pump();

    let detail = runtime
        .snapshot()
        .workbench
        .selected_message()
        .unwrap()
        .formatted_detail
        .as_ref()
        .unwrap();
    assert_eq!(detail.format, MessageDetailFormat::Json);
    assert!(detail.text.contains("formatted"));
    assert!(calls
        .lock()
        .unwrap()
        .iter()
        .any(|call| call.hook == PluginHookKind::DetailFormatter));

    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::DetailCancel("formatter cancelled".to_owned()),
        Arc::default(),
    ));
    runtime
        .command_sender()
        .send(AppCommand::RefreshMessageDetail)
        .unwrap();
    runtime.pump();
    runtime.pump();

    let detail = runtime
        .snapshot()
        .workbench
        .selected_message()
        .unwrap()
        .formatted_detail
        .as_ref()
        .unwrap();
    assert_eq!(detail.format, MessageDetailFormat::PlainText);
    assert!(!detail.diagnostics.is_empty());
}

#[test]
fn detail_transform_routes_a_save_request_to_the_host() {
    let mut snapshot = sample_snapshot(ThemeMode::System);
    enable_hook(
        &mut snapshot,
        "org.correomqtt.plugins.json-format",
        PluginHookKind::DetailTransform,
        "#",
    );
    let mut runtime = AppRuntime::with_snapshot(snapshot);
    runtime.attach_plugin_hook_executor(MockHooks::new(MockBehavior::DetailSave, Arc::default()));
    runtime
        .command_sender()
        .send(AppCommand::SelectDetailTransform(Some(
            "org.correomqtt.plugins.json-format".to_owned(),
        )))
        .unwrap();

    runtime.pump();

    assert_eq!(
        runtime.take_plugin_save_payload(),
        Some(PluginSavePayload {
            suggested_file_name: "transformed.bin".to_owned(),
            bytes: b"saved".to_vec(),
            content_type: Some("application/octet-stream".to_owned()),
        })
    );
    assert_eq!(runtime.take_plugin_save_payload(), None);

    runtime
        .command_sender()
        .send(AppCommand::RefreshMessageDetail)
        .unwrap();
    runtime.pump();

    assert_eq!(runtime.take_plugin_save_payload(), None);
}

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

#[test]
fn full_incoming_plugin_queue_returns_message_for_fallback_processing() {
    let (_, dispatch) = saturated_incoming_plugin_dispatch(false, "bridge/fallback");
    let super::plugins::IncomingPluginDispatch::Continue { event, diagnostics } = dispatch else {
        panic!("full queue must return the message for normal processing");
    };
    let crate::MqttEvent::IncomingMessage(message) = event else {
        panic!("expected incoming MQTT message");
    };
    assert_eq!(message.topic.as_str(), "bridge/fallback");
    assert!(diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("continued without topic plugin hooks")));
}

#[test]
fn full_incoming_plugin_queue_rejects_message_when_validator_cannot_run() {
    let (mut runtime, dispatch) = saturated_incoming_plugin_dispatch(true, "bridge/rejected");
    assert!(matches!(
        dispatch,
        super::plugins::IncomingPluginDispatch::Rejected
    ));

    runtime.pump();
    assert!(runtime
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic
            .message
            .contains("rejected because the plugin validator queue is full")));
}

fn saturated_incoming_plugin_dispatch(
    with_validator: bool,
    third_topic: &str,
) -> (AppRuntime, super::plugins::IncomingPluginDispatch) {
    let mut snapshot = sample_snapshot(ThemeMode::System);
    enable_hook(
        &mut snapshot,
        "org.correomqtt.plugins.base64",
        PluginHookKind::IncomingTransform,
        "bridge/#",
    );
    if with_validator {
        enable_hook(
            &mut snapshot,
            "user.advanced-validator",
            PluginHookKind::Validator,
            "bridge/#",
        );
    }
    let calls = Arc::new(Mutex::new(Vec::new()));
    let executor = Arc::new(MockHooks::new(
        MockBehavior::IncomingSlow(Duration::from_secs(1)),
        calls.clone(),
    ));
    let mut runtime = AppRuntime::with_snapshot(snapshot);
    runtime.plugin_hooks = executor.clone();
    runtime.incoming_plugin_worker =
        Some(super::incoming_plugins::IncomingPluginWorker::start_with_capacity(executor, 1));
    let connection_id = runtime.snapshot().connections[2].id;

    assert!(matches!(
        runtime.queue_incoming_hook_job(&incoming_event(connection_id, "bridge/first")),
        super::plugins::IncomingPluginDispatch::Queued
    ));
    let deadline = Instant::now() + Duration::from_secs(1);
    while calls.lock().unwrap().is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!calls.lock().unwrap().is_empty());
    assert!(matches!(
        runtime.queue_incoming_hook_job(&incoming_event(connection_id, "bridge/second")),
        super::plugins::IncomingPluginDispatch::Queued
    ));
    let dispatch = runtime.queue_incoming_hook_job(&incoming_event(connection_id, third_topic));
    (runtime, dispatch)
}

fn incoming_event(connection_id: correo_mqtt::ConnectionId, topic: &str) -> crate::MqttEvent {
    crate::MqttEvent::IncomingMessage(IncomingMessage {
        connection_id,
        topic: TopicName::new(topic).unwrap(),
        payload: b"payload".to_vec(),
        qos: Qos::AtMostOnce,
        retain: false,
        duplicate: false,
        packet_id: None,
    })
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
