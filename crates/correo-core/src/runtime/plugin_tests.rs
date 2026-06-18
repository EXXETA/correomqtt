use std::sync::{Arc, Mutex};

use correo_mqtt::{PublishRequest, Qos};

use crate::mqtt::test_support::{connection_options, connection_state, pump_until, FakeFactory};
use crate::{
    sample_snapshot, AppCommand, AppRuntime, ConnectionState, FormattedMessageDetail,
    MessageDetailFormat, MessageTransform, MqttCommand, PluginHookCall, PluginHookError,
    PluginHookExecutor, PluginHookInput, PluginHookKind, PluginHookOutput, PluginHookStatus,
    PluginMessage, PluginStatus, PluginValidation, ThemeMode,
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
        .send(AppCommand::Mqtt(MqttCommand::Connect {
            options: connection_options(connection_id),
        }))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(MqttCommand::Publish {
            connection_id,
            request: PublishRequest::new("bridge/raw", b"online".to_vec(), Qos::AtMostOnce, false)
                .unwrap(),
        }))
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
    runtime.attach_plugin_hook_executor(MockHooks::new(
        MockBehavior::ValidatorBlock("payload missing required text".to_owned()),
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
        .send(AppCommand::Mqtt(MqttCommand::Connect {
            options: connection_options(connection_id),
        }))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(MqttCommand::Publish {
            connection_id,
            request: PublishRequest::new("bridge/raw", b"online".to_vec(), Qos::AtMostOnce, false)
                .unwrap(),
        }))
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
                        && diagnostic.severity == crate::PluginDiagnosticSeverity::Error
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

#[derive(Debug)]
struct MockHooks {
    behavior: MockBehavior,
    calls: Arc<Mutex<Vec<PluginHookCall>>>,
}

impl MockHooks {
    fn new(behavior: MockBehavior, calls: Arc<Mutex<Vec<PluginHookCall>>>) -> Self {
        Self { behavior, calls }
    }
}

impl PluginHookExecutor for MockHooks {
    fn execute(&self, call: PluginHookCall) -> Result<PluginHookOutput, PluginHookError> {
        self.calls.lock().unwrap().push(call.clone());
        match (&self.behavior, call.hook, call.input) {
            (
                MockBehavior::OutgoingReplace(payload),
                PluginHookKind::OutgoingTransform,
                PluginHookInput::Message(mut message),
            ) => {
                message.payload = payload.clone();
                Ok(PluginHookOutput::MessageTransform(
                    MessageTransform::Replace(message),
                ))
            }
            (
                MockBehavior::ValidatorBlock(message),
                PluginHookKind::Validator,
                PluginHookInput::Message(_),
            ) => Ok(PluginHookOutput::Validation(PluginValidation::Block {
                message: message.clone(),
            })),
            (
                MockBehavior::ValidatorWarning(message),
                PluginHookKind::Validator,
                PluginHookInput::Message(_),
            ) => Ok(PluginHookOutput::Validation(PluginValidation::Warning {
                message: message.clone(),
            })),
            (
                MockBehavior::IncomingError(message),
                PluginHookKind::IncomingTransform,
                PluginHookInput::Message(_),
            ) => Err(PluginHookError::failed(message.clone())),
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
            (_, _, PluginHookInput::Message(message)) => Ok(PluginHookOutput::MessageTransform(
                MessageTransform::Replace(PluginMessage { ..message }),
            )),
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
}

#[derive(Debug)]
enum MockBehavior {
    OutgoingReplace(Vec<u8>),
    ValidatorBlock(String),
    ValidatorWarning(String),
    IncomingError(String),
    DetailFormat(String),
    DetailCancel(String),
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
