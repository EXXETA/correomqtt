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
    assert!(matches!(
        settings.keyring_state,
        KeyringState::Available | KeyringState::Unavailable
    ));

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
