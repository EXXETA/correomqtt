use std::path::{Path, PathBuf};

use correo_mqtt::{IncomingMessage, Qos, Subscription, TopicName};
use correo_storage::current::{MessageType, PublishStatus, Qos as StorageQos};
use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::MigrationPreview;

use super::{AppEvent, AppModel};
use crate::{
    startup_state_from_migration, AppCommand, ConnectionBadge, ConnectionState, ConnectionSurface,
    Diagnostic, GlobalSettingField, GlobalSettingFlag, KeyringState, LegacyMigrationStatus,
    MigrationFailureStage, MigrationRecoveryCommand, MigrationRecoveryCompletion,
    MigrationRecoveryCounts, MigrationRecoveryEvent, MigrationRecoveryFailure,
    MigrationRecoverySnapshot, MigrationRecoveryState, MqttCommand, MqttEvent, QosLevel,
    StartupState, ThemeMode, TransferSection, Workspace,
};

fn storage_fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../correo-storage/tests/fixtures")
        .join(path)
}

fn user_connection_ids(model: &AppModel) -> Vec<correo_mqtt::ConnectionId> {
    model
        .snapshot()
        .connections
        .iter()
        .filter(|connection| !connection.immutable)
        .map(|connection| connection.id)
        .collect()
}

fn user_connection_count(model: &AppModel) -> usize {
    user_connection_ids(model).len()
}

fn first_user_connection(model: &AppModel) -> &crate::ConnectionSummary {
    model
        .snapshot()
        .connections
        .iter()
        .find(|connection| !connection.immutable)
        .expect("user connection exists")
}

#[test]
fn applies_connection_events_to_snapshot() {
    let mut model = AppModel::default();
    let connection_id = user_connection_ids(&model)[1];

    model.apply_event(AppEvent::ConnectionOpened { connection_id });

    assert_eq!(model.snapshot().active_connection, Some(connection_id));
    assert_eq!(user_connection_count(&model), 4);
    assert_eq!(
        model
            .snapshot()
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .expect("connection exists")
            .state,
        ConnectionState::Connected
    );

    model.apply_event(AppEvent::ConnectionClosed { connection_id });

    assert_eq!(model.snapshot().active_connection, None);
    assert_eq!(
        model
            .snapshot()
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .expect("connection exists")
            .state,
        ConnectionState::Disconnected
    );
}

#[test]
fn applies_migration_recovery_events_to_snapshot() {
    let mut model = AppModel::empty();

    model.apply_event(AppEvent::MigrationRecovery(
        MigrationRecoveryEvent::LegacyDetected {
            legacy_path: "/tmp/CorreoMqtt".to_owned(),
            counts: MigrationRecoveryCounts {
                connections: 2,
                histories: 3,
                scripts: 1,
                plugin_artifacts_ignored: 4,
                warnings: 0,
                skipped_secrets: 0,
            },
            warnings: Vec::new(),
        },
    ));

    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::NeedsDecision
    );
    assert_eq!(
        model.snapshot().migration_recovery.legacy_path.as_deref(),
        Some("/tmp/CorreoMqtt")
    );
    assert_eq!(model.snapshot().migration_recovery.counts.connections, 2);
    assert_eq!(
        model.snapshot().global_settings.legacy_migration.status,
        LegacyMigrationStatus::Detected
    );
}

#[test]
fn publish_command_sets_feedback_without_recording_success_history() {
    let mut model = AppModel::default();
    let initial_count = model.snapshot().workbench.publish.history.len();

    model.apply_command(crate::AppCommand::Publish);

    assert_eq!(
        model.snapshot().workbench.publish.history.len(),
        initial_count
    );
    assert!(model
        .snapshot()
        .workbench
        .publish
        .feedback
        .as_ref()
        .is_some_and(|feedback| feedback.message.contains("queued")));
}

#[test]
fn incoming_messages_are_capped_and_subscription_counts_follow_retention() {
    let mut model = AppModel::default();
    let connection_id = user_connection_ids(&model)[0];
    model.apply_command(AppCommand::SelectConnection(connection_id));
    model.apply_event(AppEvent::Mqtt(MqttEvent::Subscribed {
        connection_id,
        subscription: Subscription::new("sensors/#", Qos::AtLeastOnce).unwrap(),
    }));

    for index in 0..1_005 {
        model.apply_event(AppEvent::Mqtt(MqttEvent::IncomingMessage(
            IncomingMessage {
                connection_id,
                topic: TopicName::new(format!("sensors/{index}")).unwrap(),
                payload: format!("payload-{index}").into_bytes(),
                qos: Qos::AtLeastOnce,
                retain: false,
                duplicate: false,
                packet_id: None,
            },
        )));
    }

    let workbench = &model.snapshot().workbench;
    assert_eq!(workbench.messages.len(), 1_000);
    assert_eq!(
        workbench
            .messages
            .first()
            .map(|message| message.topic.as_str()),
        Some("sensors/1004")
    );
    assert_eq!(
        workbench
            .messages
            .last()
            .map(|message| message.topic.as_str()),
        Some("sensors/5")
    );
    assert_eq!(workbench.subscribe.subscriptions[0].message_count, 1_000);
}

#[test]
fn incoming_message_preserves_existing_selection() {
    let mut model = AppModel::default();
    let connection_id = user_connection_ids(&model)[0];
    model.apply_command(AppCommand::SelectMessage(2));

    model.apply_event(AppEvent::Mqtt(MqttEvent::IncomingMessage(
        IncomingMessage {
            connection_id,
            topic: TopicName::new("telemetry/selection").unwrap(),
            payload: b"newest payload".to_vec(),
            qos: Qos::AtLeastOnce,
            retain: false,
            duplicate: false,
            packet_id: None,
        },
    )));

    let workbench = &model.snapshot().workbench;
    assert_eq!(
        workbench.messages.first().map(|message| message.id),
        Some(5)
    );
    assert_eq!(workbench.selected_message_id, Some(2));
}

#[test]
fn publish_history_is_capped_to_latest_rows() {
    let mut model = AppModel::default();
    let connection_id = user_connection_ids(&model)[0];
    model.apply_command(AppCommand::SelectConnection(connection_id));

    for index in 0..505 {
        model.apply_event(AppEvent::Mqtt(MqttEvent::Published {
            connection_id,
            topic: TopicName::new(format!("publish/{index}")).unwrap(),
            payload: format!("payload-{index}").into_bytes(),
            qos: Qos::AtMostOnce,
            retain: false,
            diagnostics: Vec::new(),
        }));
    }

    let history = &model.snapshot().workbench.publish.history;
    assert_eq!(history.len(), 500);
    assert_eq!(
        history.first().map(|row| row.topic.as_str()),
        Some("publish/504")
    );
    assert_eq!(
        history.last().map(|row| row.topic.as_str()),
        Some("publish/5")
    );
    assert_eq!(
        model.snapshot().workbench.publish.selected_history_id,
        history.first().map(|row| row.id)
    );
}

#[test]
fn topic_updates_refresh_core_validation_state() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::UpdatePublishTopic("alerts/#".to_owned()));
    assert!(!model.snapshot().workbench.publish.valid);
    assert!(model.snapshot().workbench.publish.validation[0].contains("wildcards"));

    model.apply_command(AppCommand::UpdateSubscribeTopic("alerts/#".to_owned()));
    assert!(model.snapshot().workbench.subscribe.valid);
    assert_eq!(
        model.snapshot().workbench.subscribe.validation,
        ["Topic filter is valid"]
    );
}

#[test]
fn copying_publish_history_message_restores_retained_state() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::CopyPublishHistoryMessageToPublishForm(3));

    assert_eq!(model.snapshot().workbench.publish.topic, "retain/config");
    assert_eq!(model.snapshot().workbench.publish.qos, QosLevel::One);
    assert!(model.snapshot().workbench.publish.retained);

    model.apply_command(AppCommand::CopyPublishHistoryMessageToPublishForm(1));
    assert_eq!(model.snapshot().workbench.publish.qos, QosLevel::One);
    assert!(!model.snapshot().workbench.publish.retained);
}

#[test]
fn copying_incoming_message_restores_retained_state() {
    let mut model = AppModel::default();
    model.apply_command(AppCommand::SetPublishRetained(false));

    model.apply_command(AppCommand::CopyIncomingMessageToPublishForm(3));

    assert_eq!(
        model.snapshot().workbench.publish.topic,
        "$SYS/broker/clients/connected"
    );
    assert_eq!(model.snapshot().workbench.publish.qos, QosLevel::Zero);
    assert!(model.snapshot().workbench.publish.retained);

    model.apply_command(AppCommand::CopyIncomingMessageToPublishForm(1));
    assert_eq!(model.snapshot().workbench.publish.qos, QosLevel::One);
    assert!(!model.snapshot().workbench.publish.retained);
}

#[test]
fn removing_publish_history_message_updates_selection() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::RemovePublishHistoryMessage(1));

    assert!(!model
        .snapshot()
        .workbench
        .publish
        .history
        .iter()
        .any(|row| row.id == 1));
    assert_eq!(
        model.snapshot().workbench.publish.selected_history_id,
        Some(2)
    );
}

#[test]
fn removing_incoming_message_updates_selection_and_subscription_counts() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::RemoveIncomingMessage(1));

    assert!(!model
        .snapshot()
        .workbench
        .messages
        .iter()
        .any(|message| message.id == 1));
    assert_eq!(model.snapshot().workbench.selected_message_id, Some(2));
    assert_eq!(
        model.snapshot().workbench.subscribe.subscriptions[0].message_count,
        127
    );
}

#[test]
fn removing_incoming_message_marks_workbench_for_persistence() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::RemoveIncomingMessage(1));
    let commands = model.drain_workbench_persistence_commands();

    assert!(commands.iter().any(|command| matches!(
        command,
        crate::HistoryPersistenceCommand::ReplaceWorkbench { workbench, .. }
            if !workbench.messages.iter().any(|message| message.id == 1)
    )));
}

#[test]
fn publish_history_removal_builds_persistence_command_from_selected_row() {
    let model = AppModel::default();

    let commands =
        model.history_commands_for_app_command(&AppCommand::RemovePublishHistoryMessage(1));

    assert_eq!(commands.len(), 1);
    let crate::HistoryPersistenceCommand::RemovePublishedMessage { message, .. } = &commands[0]
    else {
        panic!("expected remove published message command");
    };
    assert_eq!(message.topic, "telemetry/device-42/set");
    assert_eq!(
        message.payload.as_deref(),
        Some("{\n  \"target\": \"pump\",\n  \"enabled\": true\n}")
    );
    assert_eq!(message.qos, Some(StorageQos::AtLeastOnce));
    assert_eq!(message.message_type, Some(MessageType::Outgoing));
    assert_eq!(message.publish_status, Some(PublishStatus::Succeeded));
}

#[test]
fn workbench_state_is_scoped_per_selected_connection() {
    let mut model = AppModel::default();
    let user_ids = user_connection_ids(&model);
    let first = user_ids[0];
    let second = user_ids[1];

    model.apply_command(AppCommand::UpdatePublishTopic("first/topic".to_owned()));
    model.apply_command(AppCommand::UpdatePublishPayload("first payload".to_owned()));

    model.apply_command(AppCommand::SelectConnection(second));
    assert_ne!(model.snapshot().workbench.publish.topic, "first/topic");
    model.apply_command(AppCommand::UpdatePublishTopic("second/topic".to_owned()));

    model.apply_command(AppCommand::SelectConnection(first));
    assert_eq!(model.snapshot().workbench.publish.topic, "first/topic");
    assert_eq!(model.snapshot().workbench.publish.payload, "first payload");

    model.apply_command(AppCommand::SelectConnection(second));
    assert_eq!(model.snapshot().workbench.publish.topic, "second/topic");
}

#[test]
fn unsubscribe_all_dispatches_and_removes_all_subscriptions() {
    let mut model = AppModel::default();

    let direct = model
        .mqtt_commands_for_app_command(&AppCommand::UnsubscribeAll)
        .expect("unsubscribe all should build safely");
    assert_eq!(direct.len(), 3);
    assert!(direct
        .iter()
        .all(|command| matches!(command, MqttCommand::Unsubscribe { .. })));

    model.apply_command(AppCommand::UnsubscribeAll);
    assert_eq!(model.snapshot().workbench.subscribe.subscriptions.len(), 0);
}

#[test]
fn ctrl_toggles_subscription_selection() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::SelectSubscription {
        topic_filter: "telemetry/#".to_owned(),
        extend: false,
        toggle: false,
    });
    model.apply_command(AppCommand::SelectSubscription {
        topic_filter: "alerts/+".to_owned(),
        extend: false,
        toggle: true,
    });
    model.apply_command(AppCommand::SelectSubscription {
        topic_filter: "alerts/+".to_owned(),
        extend: false,
        toggle: true,
    });

    let subscriptions = &model.snapshot().workbench.subscribe.subscriptions;
    assert!(subscriptions[0].selected);
    assert!(!subscriptions[1].selected);
    assert!(!subscriptions[2].selected);
}

#[test]
fn selected_subscription_click_toggles_off_without_ctrl() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::SelectSubscription {
        topic_filter: "telemetry/#".to_owned(),
        extend: false,
        toggle: false,
    });
    model.apply_command(AppCommand::SelectSubscription {
        topic_filter: "telemetry/#".to_owned(),
        extend: false,
        toggle: true,
    });

    assert!(model
        .snapshot()
        .workbench
        .subscribe
        .subscriptions
        .iter()
        .all(|subscription| !subscription.selected));
}

