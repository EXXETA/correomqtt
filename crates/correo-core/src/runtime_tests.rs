use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{
    AppCommand, AppEvent, AppRuntime, Diagnostic, MigrationPersistenceWorker,
    MigrationRecoveryCommand, MigrationRecoveryCompletion, MigrationRecoveryState,
    SettingsPersistenceEvent, StartupState, ThemeMode,
};

#[test]
fn pump_processes_commands_without_awaiting() {
    let mut runtime = AppRuntime::new();
    runtime
        .command_sender()
        .send(AppCommand::SetThemeMode(ThemeMode::Dark))
        .unwrap();

    let report = runtime.pump();

    assert_eq!(report.commands_processed, 1);
    assert!(report.snapshot_changed);
    assert_eq!(runtime.snapshot().theme_mode, ThemeMode::Dark);
}

#[test]
fn pump_reports_changed_when_commands_restore_the_original_snapshot() {
    let mut runtime = AppRuntime::new();
    runtime
        .command_sender()
        .send(AppCommand::SearchConnections("mqtt".to_owned()))
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::SearchConnections(String::new()))
        .unwrap();

    let report = runtime.pump();

    assert_eq!(report.commands_processed, 2);
    assert!(report.snapshot_changed);
    assert!(runtime.snapshot().connection_filter.is_empty());
}

#[test]
fn pump_reports_changed_for_built_in_broker_settings_warning() {
    let mut runtime = AppRuntime::new();
    runtime
        .command_sender()
        .send(AppCommand::OpenConnectionSettings(
            crate::built_in_broker_connection_id(),
        ))
        .unwrap();

    let report = runtime.pump();

    assert_eq!(report.commands_processed, 1);
    assert!(report.snapshot_changed);
    assert!(runtime
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic
            .message
            .contains("Correo Broker connection is managed automatically.")));
}

#[test]
fn pump_redacts_service_diagnostics() {
    let mut runtime = AppRuntime::new();
    runtime
        .event_sender()
        .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
            "auth failed: password:open-sesame",
        )))
        .unwrap();

    runtime.pump();

    let message = &runtime.snapshot().diagnostics[0].message;
    assert!(!message.contains("open-sesame"));
    assert!(message.contains("[REDACTED]"));
}

#[test]
fn saved_new_connection_reaches_config_store() {
    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    let temp = tempfile::tempdir().unwrap();
    let mut runtime = AppRuntime::new();
    runtime.attach_settings_worker(crate::SettingsPersistenceWorker::start(temp.path()));

    runtime
        .command_sender()
        .send(AppCommand::AddConnection)
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::UpdateConnectionSetting {
            field: crate::ConnectionSettingField::Host,
            value: "broker.local".to_owned(),
        })
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::SaveConnectionSettings)
        .unwrap();

    runtime.pump();
    assert_eq!(
        runtime.recv_settings_event_timeout(Duration::from_secs(3)),
        Some(SettingsPersistenceEvent::Saved)
    );
    runtime.pump();

    let config = correo_storage::current::ConfigStore::new(temp.path())
        .load()
        .expect("saved connection config");
    assert_eq!(config.connections.len(), 1);
    assert_eq!(config.connections[0].url, "broker.local");
    assert_eq!(config.connections[0].name, "New connection");
}

#[test]
fn rejected_connection_settings_save_is_not_dispatched() {
    // No settings worker attached: a dispatch attempt raises the
    // "worker is not running" warning, a skipped dispatch does not.
    let mut runtime = AppRuntime::new();

    runtime
        .command_sender()
        .send(AppCommand::AddConnection)
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::SaveConnectionSettings)
        .unwrap();
    runtime.pump();
    runtime.pump();
    assert!(
        !worker_missing_warning(&runtime),
        "invalid draft save must not be dispatched to the persistence worker"
    );

    runtime
        .command_sender()
        .send(AppCommand::UpdateConnectionSetting {
            field: crate::ConnectionSettingField::Host,
            value: "broker.local".to_owned(),
        })
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::SaveConnectionSettings)
        .unwrap();
    runtime.pump();
    runtime.pump();
    assert!(
        worker_missing_warning(&runtime),
        "valid save should reach the dispatch path"
    );
}

fn worker_missing_warning(runtime: &AppRuntime) -> bool {
    runtime
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("worker is not running"))
}

#[test]
fn migration_worker_advances_recovery_flow_to_complete() {
    // Hermetic: the real OS keyring may hold a Java master password, which
    // would auto-unlock secrets and skip the expected password step.
    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    let temp = tempfile::tempdir().unwrap();
    let legacy_path = storage_fixture("legacy_profile").display().to_string();
    let mut runtime = AppRuntime::with_startup_state(StartupState::legacy_migration_detected(
        ThemeMode::Dark,
        legacy_path,
    ));
    runtime.attach_migration_worker(MigrationPersistenceWorker::start(temp.path()));

    runtime
        .command_sender()
        .send(AppCommand::MigrationRecovery(
            MigrationRecoveryCommand::ChooseMigrate,
        ))
        .unwrap();
    runtime.pump();
    assert_eq!(
        runtime.snapshot().migration_recovery.state,
        MigrationRecoveryState::CreatingBackup
    );

    assert!(
        runtime.wait_for_migration(Duration::from_secs(3), |runtime| {
            runtime.snapshot().migration_recovery.state == MigrationRecoveryState::NeedsPassword
        }),
        "migration backup completion"
    );
    assert_eq!(
        runtime.snapshot().migration_recovery.state,
        MigrationRecoveryState::NeedsPassword
    );
    assert!(runtime.snapshot().migration_recovery.backup_name.is_some());

    runtime
        .command_sender()
        .send(AppCommand::MigrationRecovery(
            MigrationRecoveryCommand::SkipSecrets,
        ))
        .unwrap();
    runtime.pump();
    assert!(
        runtime.wait_for_migration(Duration::from_secs(3), |runtime| {
            let recovery = &runtime.snapshot().migration_recovery;
            recovery.state == MigrationRecoveryState::Reviewing && recovery.counts.connections == 2
        }),
        "migration preview completion"
    );
    assert_eq!(
        runtime.snapshot().migration_recovery.state,
        MigrationRecoveryState::Reviewing
    );
    assert_eq!(runtime.snapshot().migration_recovery.counts.connections, 2);

    runtime
        .command_sender()
        .send(AppCommand::MigrationRecovery(
            MigrationRecoveryCommand::ApplyMigration,
        ))
        .unwrap();
    runtime.pump();
    assert!(
        runtime.wait_for_migration(Duration::from_secs(3), |runtime| {
            runtime.snapshot().migration_recovery.state == MigrationRecoveryState::Complete
        }),
        "migration apply completion"
    );
    assert_eq!(
        runtime.snapshot().migration_recovery.state,
        MigrationRecoveryState::Complete
    );

    assert_eq!(
        runtime
            .snapshot()
            .connections
            .iter()
            .filter(|connection| !connection.immutable)
            .count(),
        2
    );
    assert!(temp.path().join("config.json").exists());
}

#[test]
fn migration_restore_command_restores_selected_backup() {
    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    let temp = tempfile::tempdir().unwrap();
    let legacy_path = storage_fixture("legacy_profile").display().to_string();
    let mut runtime = AppRuntime::with_startup_state(StartupState::legacy_migration_detected(
        ThemeMode::Dark,
        legacy_path,
    ));
    runtime.attach_migration_worker(MigrationPersistenceWorker::start(temp.path()));

    runtime
        .command_sender()
        .send(AppCommand::MigrationRecovery(
            MigrationRecoveryCommand::ChooseMigrate,
        ))
        .unwrap();
    runtime.pump();
    assert!(
        runtime.wait_for_migration(Duration::from_secs(3), |runtime| {
            runtime.snapshot().migration_recovery.state == MigrationRecoveryState::NeedsPassword
        }),
        "migration backup completion"
    );
    assert_eq!(
        runtime.snapshot().migration_recovery.state,
        MigrationRecoveryState::NeedsPassword
    );
    runtime
        .command_sender()
        .send(AppCommand::MigrationRecovery(
            MigrationRecoveryCommand::SkipSecrets,
        ))
        .unwrap();
    runtime.pump();
    assert!(
        runtime.wait_for_migration(Duration::from_secs(3), |runtime| {
            let recovery = &runtime.snapshot().migration_recovery;
            recovery.state == MigrationRecoveryState::Reviewing && recovery.counts.connections == 2
        }),
        "migration preview completion"
    );
    assert_eq!(
        runtime.snapshot().migration_recovery.state,
        MigrationRecoveryState::Reviewing
    );
    runtime
        .command_sender()
        .send(AppCommand::MigrationRecovery(
            MigrationRecoveryCommand::ApplyMigration,
        ))
        .unwrap();
    runtime.pump();
    assert!(
        runtime.wait_for_migration(Duration::from_secs(3), |runtime| {
            runtime.snapshot().migration_recovery.state == MigrationRecoveryState::Complete
        }),
        "migration apply completion"
    );
    assert_eq!(
        runtime.snapshot().migration_recovery.state,
        MigrationRecoveryState::Complete
    );
    assert!(temp.path().join("config.json").exists());
    let restarted_snapshot = runtime.snapshot().clone();
    drop(runtime);
    let mut runtime = AppRuntime::with_snapshot(restarted_snapshot);
    runtime.attach_migration_worker(MigrationPersistenceWorker::start(temp.path()));

    runtime
        .command_sender()
        .send(AppCommand::MigrationRecovery(
            MigrationRecoveryCommand::RequestRestoreBackup,
        ))
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::MigrationRecovery(
            MigrationRecoveryCommand::ConfirmRestoreBackup,
        ))
        .unwrap();
    runtime.pump();
    assert!(
        runtime.wait_for_migration(Duration::from_secs(3), |runtime| {
            let recovery = &runtime.snapshot().migration_recovery;
            recovery.state == MigrationRecoveryState::Complete
                && recovery.completion == Some(MigrationRecoveryCompletion::RestoreSuccess)
        }),
        "migration restore completion"
    );
    assert_eq!(
        runtime.snapshot().migration_recovery.state,
        MigrationRecoveryState::Complete
    );
    assert_eq!(
        runtime.snapshot().migration_recovery.completion,
        Some(MigrationRecoveryCompletion::RestoreSuccess)
    );

    assert!(!temp.path().join("config.json").exists());
}

#[test]
fn command_sender_returns_backpressure_without_losing_the_command() {
    let runtime = AppRuntime::new();
    let sender = runtime.command_sender();
    for _ in 0..256 {
        sender
            .send(AppCommand::SetThemeMode(ThemeMode::Dark))
            .expect("commands within the fixed capacity are accepted");
    }

    let error = sender
        .send(AppCommand::SetThemeMode(ThemeMode::Light))
        .unwrap_err();
    assert!(matches!(
        error,
        crate::CommandSendError::CommandFull(command)
            if matches!(*command, AppCommand::SetThemeMode(ThemeMode::Light))
    ));
}

#[test]
fn event_sender_returns_backpressure_without_silent_loss() {
    let runtime = AppRuntime::new();
    let sender = runtime.event_sender();
    for index in 0..256 {
        sender
            .emit(AppEvent::DiagnosticRaised(Diagnostic::info(format!(
                "event {index}"
            ))))
            .expect("events within the fixed capacity are accepted");
    }

    let error = sender
        .emit(AppEvent::DiagnosticRaised(Diagnostic::info("overflow")))
        .unwrap_err();
    assert!(matches!(
        error,
        crate::CommandSendError::EventFull(event)
            if matches!(*event, AppEvent::DiagnosticRaised(_))
    ));
}

#[test]
fn bounded_pump_yields_to_a_queued_ui_command_under_event_ingress() {
    let mut runtime = AppRuntime::new();
    let events = runtime.event_sender();
    for index in 0..65 {
        events
            .emit(AppEvent::DiagnosticRaised(Diagnostic::info(format!(
                "event {index}"
            ))))
            .unwrap();
    }
    runtime
        .command_sender()
        .send(AppCommand::SetThemeMode(ThemeMode::Dark))
        .unwrap();

    let report = runtime.pump();

    assert!(
        report.events_processed < 65,
        "the per-frame event budget must yield before draining ingress"
    );
    assert_eq!(
        runtime.snapshot().theme_mode,
        ThemeMode::Dark,
        "an available UI command must progress despite ingress backlog"
    );
}

fn storage_fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../correo-storage/tests/fixtures")
        .join(path)
}
