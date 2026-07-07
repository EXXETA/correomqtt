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

fn first_user_connection<'a>(model: &'a AppModel) -> &'a crate::ConnectionSummary {
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

#[test]
fn run_script_dispatches_connect_for_disconnected_script_connection() {
    let mut model = AppModel::default();
    let disconnected_id = user_connection_ids(&model)[1];
    model.apply_command(AppCommand::SelectScriptConnection(
        disconnected_id.to_string(),
    ));

    let commands = model
        .mqtt_commands_for_app_command(&AppCommand::RunScript)
        .expect("run script should build pre-connect command");

    assert!(matches!(
        commands.as_slice(),
        [MqttCommand::Connect { options }] if options.connection_id == disconnected_id
    ));
}

#[test]
fn global_settings_commands_track_dirty_save_and_discard() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::UpdateGlobalSetting {
        field: GlobalSettingField::Language,
        value: "de_DE".to_owned(),
    });
    model.apply_command(AppCommand::SetGlobalSettingFlag {
        flag: GlobalSettingFlag::UseRegexForSearch,
        enabled: true,
    });
    model.apply_command(AppCommand::SetGlobalSettingFlag {
        flag: GlobalSettingFlag::ReduceMotion,
        enabled: true,
    });
    model.apply_command(AppCommand::SetThemeMode(ThemeMode::Dark));

    assert!(model.snapshot().global_settings.dirty);
    assert_eq!(model.snapshot().global_settings.language, "de_DE");
    assert_eq!(model.snapshot().theme_mode, ThemeMode::Dark);

    model.apply_command(AppCommand::SaveGlobalSettings);
    assert!(!model.snapshot().global_settings.dirty);

    model.apply_command(AppCommand::UpdateGlobalSetting {
        field: GlobalSettingField::KeyringBackend,
        value: "LibSecret".to_owned(),
    });
    model.apply_command(AppCommand::SetThemeMode(ThemeMode::Light));
    model.apply_command(AppCommand::DiscardGlobalSettings);

    assert_eq!(model.snapshot().global_settings.language, "de_DE");
    assert_eq!(model.snapshot().global_settings.keyring_backend, "os");
    assert!(model.snapshot().global_settings.search_use_regex);
    assert!(model.snapshot().global_settings.reduce_motion);
    assert_eq!(model.snapshot().theme_mode, ThemeMode::Dark);
    assert!(!model.snapshot().global_settings.dirty);
}

#[test]
fn global_settings_plugin_repository_commands_edit_rows() {
    let mut model = AppModel::empty();

    model.apply_command(AppCommand::AddPluginRepository);
    assert_eq!(
        model.snapshot().global_settings.plugin_repositories.len(),
        1
    );
    assert_eq!(
        model.snapshot().global_settings.plugin_repositories[0].id,
        "custom-1"
    );
    assert!(model.snapshot().global_settings.dirty);

    model.apply_command(AppCommand::UpdatePluginRepository {
        index: 0,
        url: "https://example.invalid/plugins.json".to_owned(),
    });
    assert_eq!(
        model.snapshot().global_settings.plugin_repositories[0].url,
        "https://example.invalid/plugins.json"
    );

    model.apply_command(AppCommand::SaveGlobalSettings);
    assert!(!model.snapshot().global_settings.dirty);

    model.apply_command(AppCommand::RemovePluginRepository { index: 0 });
    assert!(model
        .snapshot()
        .global_settings
        .plugin_repositories
        .is_empty());
    assert!(model.snapshot().global_settings.dirty);
}

#[test]
fn connect_command_queues_service_work_without_marking_open() {
    let mut model = AppModel::default();
    let connection_id = user_connection_ids(&model)[2];
    let initial_active_connection = model.snapshot().active_connection;

    model.apply_command(AppCommand::Connect(connection_id));

    assert_eq!(
        model.snapshot().active_connection,
        initial_active_connection
    );
    assert_eq!(
        model
            .snapshot()
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .expect("connection exists")
            .state,
        ConnectionState::Connecting
    );
}

#[test]
fn add_connection_opens_settings_draft_and_save_adds_profile() {
    let mut model = AppModel::empty();

    model.apply_command(AppCommand::AddConnection);

    assert_eq!(model.snapshot().active_workspace, Workspace::Connections);
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Settings
    );
    assert_eq!(model.snapshot().selected_connection, None);
    assert_eq!(
        model.snapshot().connection_settings.profile_name,
        "New connection"
    );
    assert!(model.snapshot().connection_settings.dirty);
    assert!(!model.snapshot().connection_settings.valid);
    assert!(model
        .snapshot()
        .connection_settings
        .validation_errors
        .iter()
        .any(|error| error == "Host is required"));

    model.apply_command(AppCommand::SaveConnectionSettings);
    assert!(model
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Host is required"));

    model.apply_command(AppCommand::UpdateConnectionSetting {
        field: crate::ConnectionSettingField::Host,
        value: "localhost".to_owned(),
    });
    assert!(model.snapshot().connection_settings.valid);

    model.apply_command(AppCommand::SaveConnectionSettings);

    let connection_id = model
        .snapshot()
        .selected_connection
        .expect("saved draft should become selected");
    let connection = model
        .snapshot()
        .selected_connection()
        .expect("saved draft should be visible in launcher");
    assert_eq!(user_connection_count(&model), 1);
    assert_eq!(connection.name, "New connection");
    assert_eq!(connection.endpoint, "localhost:1883");
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert!(!model.snapshot().connection_settings.dirty);
    assert!(model
        .mqtt_commands_for_app_command(&AppCommand::Connect(connection_id))
        .expect("new profile should build connect command")
        .iter()
        .any(|command| matches!(command, MqttCommand::Connect { .. })));
}

#[test]
fn transfer_commands_focus_the_requested_section() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::ExportConnections);
    assert_eq!(model.snapshot().active_workspace, Workspace::Connections);
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Transfer
    );
    assert_eq!(
        model.snapshot().transfer.active_section,
        TransferSection::Export
    );

    model.apply_command(AppCommand::ImportMessages);
    assert_eq!(model.snapshot().active_workspace, Workspace::Connections);
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert!(model.snapshot().workbench.publish.feedback.is_some());

    model.apply_command(AppCommand::ImportConnections);
    assert_eq!(model.snapshot().active_workspace, Workspace::Connections);
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Transfer
    );
    assert_eq!(
        model.snapshot().transfer.active_section,
        TransferSection::Import
    );
}

#[test]
fn message_import_path_loads_cqm_into_publish_form() {
    let mut model = AppModel::default();
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("message.cqm");
    std::fs::write(
        &path,
        r#"{
            "topic": "import/topic",
            "payload": "imported payload",
            "qos": 2,
            "retained": true
        }"#,
    )
    .expect("write cqm fixture");

    model.apply_command(AppCommand::ImportMessagesFromPath(path));

    let publish = &model.snapshot().workbench.publish;
    assert_eq!(publish.topic, "import/topic");
    assert_eq!(publish.payload, "imported payload");
    assert_eq!(publish.qos, QosLevel::Two);
    assert!(publish.retained);
    assert!(publish.valid);
    assert!(publish
        .feedback
        .as_ref()
        .is_some_and(|feedback| feedback.message.contains("Loaded .cqm")));
}

#[test]
fn diagnostic_events_are_redacted_before_snapshot_exposure() {
    let mut model = AppModel::empty();

    model.apply_event(AppEvent::DiagnosticRaised(Diagnostic::error(
        "mqtt auth failed password=super-secret token:abcd",
    )));

    let message = &model.snapshot().diagnostics[0].message;
    assert!(message.contains("[REDACTED]"));
    assert!(!message.contains("super-secret"));
    assert!(!message.contains("abcd"));
}

#[test]
fn migrated_fixture_opens_workbench_and_settings_without_secret_values() {
    let profile = LegacyProfile::read_from(storage_fixture("legacy_profile")).unwrap();
    let preview = MigrationPreview::from_legacy_profile(profile).unwrap();
    let state = startup_state_from_migration(preview, ThemeMode::Dark);
    let mut model = AppModel::with_startup_state(state);

    assert_eq!(model.snapshot().theme_mode, ThemeMode::Light);
    assert_eq!(user_connection_count(&model), 2);

    let first = first_user_connection(&model);
    assert_eq!(first.name, "Synthetic Local Broker");
    assert_eq!(first.endpoint, "localhost:1883");
    assert_eq!(first.mqtt_version, "MQTT v5");
    assert!(first.badges.contains(&ConnectionBadge::Credentials));
    assert!(first.badges.contains(&ConnectionBadge::Proxy));
    assert!(first.badges.contains(&ConnectionBadge::Lwt));
    assert_eq!(first.recent_subscriptions, 2);
    assert_eq!(first.recent_messages, 1);

    assert_eq!(
        model.snapshot().workbench.publish.topic_history,
        ["sensors/temperature", "alerts/status"]
    );
    assert_eq!(
        model.snapshot().workbench.subscribe.topic_history,
        ["sensors/#", "alerts/status"]
    );
    assert!(model
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic
            .message
            .contains("Unsupported legacy field ignored")));

    let first_id = first_user_connection(&model).id;
    model.apply_command(AppCommand::OpenConnectionSettings(first_id));
    let settings = &model.snapshot().connection_settings;
    assert_eq!(settings.profile_name, "Synthetic Local Broker");
    assert_eq!(settings.host, "localhost");
    assert_eq!(settings.port, "1883");
    assert_eq!(settings.mqtt_version, "MQTT v5");
    assert_eq!(settings.proxy_mode, "SSH");
    assert!(settings.lwt_enabled);
    assert_eq!(settings.lwt_topic, "status/local-broker-01");
    assert_eq!(settings.keyring_state, KeyringState::Available);

    let exposed = format!("{:?}", model.snapshot());
    assert!(!exposed.contains("synthetic-mqtt-password"));
    assert!(!exposed.contains("synthetic-ssh-password"));
    assert!(!exposed.contains("synthetic-keystore-password"));
}

#[test]
fn first_run_without_legacy_data_keeps_connections_workspace_available() {
    let state = StartupState::empty(
        ThemeMode::Light,
        Diagnostic::info("No existing CorreoMQTT config found; empty workspace ready."),
    );
    let model = AppModel::with_startup_state(state);

    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::NotDetected
    );
    assert!(!model.snapshot().migration_recovery.blocks_normal_shell());
}

#[test]
fn legacy_detection_blocks_launcher_until_user_choice() {
    let state = StartupState::legacy_migration_detected(
        ThemeMode::Dark,
        "/home/user/.correomqtt".to_owned(),
    );
    let mut model = AppModel::with_startup_state(state);

    assert!(model.snapshot().migration_recovery.blocks_normal_shell());
    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::NeedsDecision
    );

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::StartEmptyProfile,
    ));
    assert!(
        model
            .snapshot()
            .migration_recovery
            .empty_profile_confirmation_open
    );

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::ConfirmStartEmptyProfile,
    ));
    assert!(!model.snapshot().migration_recovery.blocks_normal_shell());
    assert_eq!(
        model.snapshot().global_settings.legacy_migration.status,
        LegacyMigrationStatus::Skipped
    );
}

#[test]
fn failure_after_write_offers_restore_and_settings_data_status() {
    let mut recovery = MigrationRecoverySnapshot::detected("/home/user/.correomqtt");
    recovery.backup_name = Some("migration-backup-123".to_owned());
    recovery.backup_path_hint = Some("/tmp/backups/migration-backup-123".to_owned());
    let mut model = AppModel::with_snapshot(crate::AppSnapshot {
        migration_recovery: recovery,
        ..crate::AppSnapshot::empty()
    });

    model.apply_event(AppEvent::MigrationRecovery(
        MigrationRecoveryEvent::ApplyFailed {
            failure: MigrationRecoveryFailure {
                stage: MigrationFailureStage::AfterWrite,
                message: "config write failed after backup".to_owned(),
            },
        },
    ));

    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::Failed
    );
    assert_eq!(
        model.snapshot().global_settings.legacy_migration.status,
        LegacyMigrationStatus::Failed
    );
    assert!(
        model
            .snapshot()
            .global_settings
            .legacy_migration
            .restore_available
    );

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::RequestRestoreBackup,
    ));
    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::RestoreConfirm
    );
}

#[test]
fn partial_success_keeps_recovery_complete_until_connections_opened() {
    let mut model = AppModel::with_snapshot(crate::AppSnapshot {
        migration_recovery: MigrationRecoverySnapshot::detected("/home/user/.correomqtt"),
        ..crate::AppSnapshot::empty()
    });

    model.apply_event(AppEvent::MigrationRecovery(
        MigrationRecoveryEvent::ApplyCompleted {
            completion: MigrationRecoveryCompletion::PartialSuccess,
            diagnostics: Vec::new(),
        },
    ));

    assert!(model.snapshot().migration_recovery.blocks_normal_shell());
    assert_eq!(
        model.snapshot().global_settings.legacy_migration.status,
        LegacyMigrationStatus::PartialSuccess
    );

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::OpenConnections,
    ));
    assert!(!model.snapshot().migration_recovery.blocks_normal_shell());
}
