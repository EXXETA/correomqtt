use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

use correo_mqtt::{PublishRequest, Qos};

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

