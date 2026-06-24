use crate::{
    AppCommand, Diagnostic, LegacyMigrationStatus, MigrationApplyStage,
    MigrationDiagnosticCategory, MigrationPasswordError, MigrationRecoveryCommand,
    MigrationRecoveryCompletion, MigrationRecoveryDiagnostic, MigrationRecoveryEvent,
    MigrationRecoveryFailure, MigrationRecoverySnapshot, MigrationRecoveryState, SettingsSection,
    StartupState, Workspace,
};

use super::AppModel;

impl AppModel {
    pub(super) fn apply_migration_recovery_command(&mut self, command: &AppCommand) -> bool {
        let AppCommand::MigrationRecovery(command) = command else {
            return false;
        };
        match command {
            MigrationRecoveryCommand::ChooseMigrate => self.start_backup(),
            MigrationRecoveryCommand::StartEmptyProfile => {
                self.snapshot
                    .migration_recovery
                    .empty_profile_confirmation_open = true;
            }
            MigrationRecoveryCommand::CancelEmptyProfile => {
                self.snapshot
                    .migration_recovery
                    .empty_profile_confirmation_open = false;
            }
            MigrationRecoveryCommand::ConfirmStartEmptyProfile => self.start_empty_profile(),
            MigrationRecoveryCommand::SubmitPassword => self.unlock_secrets(),
            MigrationRecoveryCommand::SkipSecrets => self.skip_secrets(),
            MigrationRecoveryCommand::SelectMigrationItem { item_id, selected } => {
                self.select_migration_item(item_id, *selected);
            }
            MigrationRecoveryCommand::ApplyMigration => self.start_apply(),
            MigrationRecoveryCommand::Retry => self.retry_migration(),
            MigrationRecoveryCommand::RequestRestoreBackup => self.request_restore(),
            MigrationRecoveryCommand::CancelRestoreBackup => self.cancel_restore(),
            MigrationRecoveryCommand::ConfirmRestoreBackup => self.start_restore(),
            MigrationRecoveryCommand::OpenDiagnostics => {
                self.snapshot.active_workspace = Workspace::Diagnostics;
            }
            MigrationRecoveryCommand::OpenSettingsData => {
                self.snapshot.active_workspace = Workspace::Settings;
                self.snapshot.global_settings.selected_section = SettingsSection::Data;
            }
            MigrationRecoveryCommand::OpenConnections => self.open_connections_after_migration(),
        }
        true
    }

    pub(super) fn apply_migration_recovery_event(&mut self, event: MigrationRecoveryEvent) {
        match event {
            MigrationRecoveryEvent::NoLegacyData => {
                self.snapshot.migration_recovery = MigrationRecoverySnapshot::default();
            }
            MigrationRecoveryEvent::LegacyDetected {
                legacy_path,
                counts,
                warnings,
            } => {
                let mut recovery = MigrationRecoverySnapshot::detected(legacy_path);
                recovery.counts = counts;
                recovery.counts.warnings = warnings.len();
                recovery.warnings = warnings;
                self.snapshot.migration_recovery = recovery;
                self.refresh_legacy_settings(LegacyMigrationStatus::Detected);
            }
            MigrationRecoveryEvent::DetectionFailed { message } => {
                self.snapshot.migration_recovery.state = MigrationRecoveryState::Failed;
                self.snapshot.migration_recovery.failure = Some(MigrationRecoveryFailure {
                    stage: crate::MigrationFailureStage::BeforeWrite,
                    message: message.clone(),
                });
                self.push_diagnostic(Diagnostic::error(message));
                self.refresh_legacy_settings(LegacyMigrationStatus::Failed);
            }
            MigrationRecoveryEvent::BackupStarted => self.start_backup(),
            MigrationRecoveryEvent::BackupCreated {
                backup_name,
                backup_path_hint,
            } => self.backup_created(backup_name, backup_path_hint),
            MigrationRecoveryEvent::BackupFailed { message } => self.backup_failed(message),
            MigrationRecoveryEvent::PasswordNeeded => {
                self.snapshot.migration_recovery.state = MigrationRecoveryState::NeedsPassword;
            }
            MigrationRecoveryEvent::PasswordRejected => {
                self.snapshot.migration_recovery.password_error =
                    Some(MigrationPasswordError::WrongPassword);
            }
            MigrationRecoveryEvent::UnsupportedEncryption => {
                self.snapshot.migration_recovery.password_error =
                    Some(MigrationPasswordError::UnsupportedEncryption);
            }
            MigrationRecoveryEvent::SecretsUnlocked { imported_count } => {
                self.snapshot.migration_recovery.state = MigrationRecoveryState::Reviewing;
                self.snapshot.migration_recovery.password_error = None;
                self.snapshot.migration_recovery.diagnostics.push(
                    MigrationRecoveryDiagnostic::info(
                        MigrationDiagnosticCategory::Secret,
                        format!("{imported_count} secret(s) prepared for OS keyring import."),
                    ),
                );
            }
            MigrationRecoveryEvent::SecretsSkipped { skipped_count } => {
                self.snapshot.migration_recovery.counts.skipped_secrets = skipped_count;
                self.skip_secrets();
            }
            MigrationRecoveryEvent::ReviewReady {
                counts,
                rows,
                warnings,
            } => {
                self.snapshot.migration_recovery.state = MigrationRecoveryState::Reviewing;
                self.snapshot.migration_recovery.counts = counts;
                self.snapshot.migration_recovery.rows = rows;
                self.snapshot.migration_recovery.counts.warnings = warnings.len();
                self.snapshot.migration_recovery.warnings = warnings;
            }
            MigrationRecoveryEvent::ApplyStarted => self.start_apply(),
            MigrationRecoveryEvent::ApplyStageChanged { stage } => {
                self.snapshot.migration_recovery.current_stage = Some(stage);
            }
            MigrationRecoveryEvent::ApplyCompleted {
                completion,
                diagnostics,
            } => self.apply_completed(completion, diagnostics),
            MigrationRecoveryEvent::ApplyFailed { failure } => self.apply_failed(failure),
            MigrationRecoveryEvent::RestoreStarted => self.start_restore(),
            MigrationRecoveryEvent::RestoreCompleted => self.restore_completed(),
            MigrationRecoveryEvent::RestoreFailed { message } => self.restore_failed(message),
        }
    }

    fn start_backup(&mut self) {
        let recovery = &mut self.snapshot.migration_recovery;
        recovery.state = MigrationRecoveryState::CreatingBackup;
        recovery.backup_status = "Creating backup before migration...".to_owned();
        recovery.failure = None;
        recovery.completion = None;
        recovery.current_stage = None;
    }

    fn backup_created(&mut self, backup_name: String, backup_path_hint: String) {
        let recovery = &mut self.snapshot.migration_recovery;
        recovery.backup_name = Some(backup_name.clone());
        recovery.backup_path_hint = Some(backup_path_hint);
        recovery.backup_status = format!("Backup created: {backup_name}");
        recovery.state = MigrationRecoveryState::NeedsPassword;
        recovery.diagnostics.push(MigrationRecoveryDiagnostic::info(
            MigrationDiagnosticCategory::Backup,
            "Backup created before migration.",
        ));
        self.refresh_legacy_settings(LegacyMigrationStatus::Detected);
    }

    fn backup_failed(&mut self, message: String) {
        let recovery = &mut self.snapshot.migration_recovery;
        recovery.state = MigrationRecoveryState::Failed;
        recovery.failure = Some(MigrationRecoveryFailure {
            stage: crate::MigrationFailureStage::Backup,
            message: message.clone(),
        });
        recovery
            .diagnostics
            .push(MigrationRecoveryDiagnostic::error(
                MigrationDiagnosticCategory::Backup,
                message,
            ));
        self.refresh_legacy_settings(LegacyMigrationStatus::Failed);
    }

    fn unlock_secrets(&mut self) {
        self.snapshot.migration_recovery.state = MigrationRecoveryState::Reviewing;
        self.snapshot.migration_recovery.password_error = None;
    }

    fn skip_secrets(&mut self) {
        let recovery = &mut self.snapshot.migration_recovery;
        recovery.state = MigrationRecoveryState::Reviewing;
        recovery.secrets_skipped = true;
        recovery.password_error = None;
    }

    fn select_migration_item(&mut self, item_id: &str, selected: bool) {
        if let Some(row) = self
            .snapshot
            .migration_recovery
            .rows
            .iter_mut()
            .find(|row| row.id == item_id)
        {
            row.selected = selected;
        }
    }

    fn start_apply(&mut self) {
        let recovery = &mut self.snapshot.migration_recovery;
        recovery.state = MigrationRecoveryState::Applying;
        recovery.current_stage = Some(MigrationApplyStage::BackupVerified);
        recovery.failure = None;
    }

    fn apply_completed(
        &mut self,
        completion: MigrationRecoveryCompletion,
        diagnostics: Vec<MigrationRecoveryDiagnostic>,
    ) {
        let recovery = &mut self.snapshot.migration_recovery;
        recovery.state = MigrationRecoveryState::Complete;
        recovery.completion = Some(completion);
        recovery.current_stage = Some(MigrationApplyStage::DiagnosticsRecorded);
        recovery.diagnostics.extend(diagnostics);
        let status = match completion {
            MigrationRecoveryCompletion::Success => LegacyMigrationStatus::Complete,
            MigrationRecoveryCompletion::PartialSuccess => LegacyMigrationStatus::PartialSuccess,
            MigrationRecoveryCompletion::RestoreSuccess => LegacyMigrationStatus::Restored,
        };
        self.refresh_legacy_settings(status);
    }

    pub(super) fn apply_migrated_startup_state(
        &mut self,
        state: StartupState,
        completion: MigrationRecoveryCompletion,
        diagnostics: Vec<MigrationRecoveryDiagnostic>,
    ) {
        let recovery = self.snapshot.migration_recovery.clone();
        self.snapshot = state.snapshot;
        self.connection_settings = state.connection_settings;
        self.storage_connection_ids = state.storage_connection_ids;
        self.saved_global_settings = self.snapshot.global_settings.clone();
        self.saved_theme_mode = self.snapshot.theme_mode.clone();
        self.sync_built_in_broker_connection();
        self.snapshot.migration_recovery = recovery;
        self.apply_completed(completion, diagnostics);
    }

    fn apply_failed(&mut self, failure: MigrationRecoveryFailure) {
        self.snapshot.migration_recovery.state = MigrationRecoveryState::Failed;
        self.snapshot.migration_recovery.failure = Some(failure.clone());
        self.snapshot
            .migration_recovery
            .diagnostics
            .push(MigrationRecoveryDiagnostic::error(
                MigrationDiagnosticCategory::Migration,
                failure.message,
            ));
        self.refresh_legacy_settings(LegacyMigrationStatus::Failed);
    }

    fn retry_migration(&mut self) {
        self.snapshot.migration_recovery.failure = None;
        self.snapshot.migration_recovery.state = MigrationRecoveryState::NeedsDecision;
    }

    fn request_restore(&mut self) {
        if self.snapshot.migration_recovery.backup_name.is_some() {
            self.snapshot.migration_recovery.state = MigrationRecoveryState::RestoreConfirm;
            return;
        }
        let settings = &self.snapshot.global_settings.legacy_migration;
        if settings.restore_available {
            let mut recovery = MigrationRecoverySnapshot::default();
            recovery.state = MigrationRecoveryState::RestoreConfirm;
            recovery.legacy_path = settings.legacy_path_hint.clone();
            recovery.backup_name = settings.backup_name.clone();
            recovery.backup_path_hint = settings.backup_path_hint.clone();
            recovery.backup_status = settings
                .backup_name
                .as_ref()
                .map(|name| format!("Backup created: {name}"))
                .unwrap_or_else(|| "Backup selected".to_owned());
            self.snapshot.migration_recovery = recovery;
        }
    }

    fn cancel_restore(&mut self) {
        self.snapshot.migration_recovery.state = MigrationRecoveryState::Failed;
    }

    fn start_restore(&mut self) {
        self.snapshot.migration_recovery.state = MigrationRecoveryState::Restoring;
        self.snapshot.migration_recovery.current_stage = None;
    }

    fn restore_completed(&mut self) {
        self.apply_completed(MigrationRecoveryCompletion::RestoreSuccess, Vec::new());
    }

    fn restore_failed(&mut self, message: String) {
        self.snapshot.migration_recovery.state = MigrationRecoveryState::Failed;
        self.snapshot.migration_recovery.failure = Some(MigrationRecoveryFailure {
            stage: crate::MigrationFailureStage::Restore,
            message: message.clone(),
        });
        self.snapshot
            .migration_recovery
            .diagnostics
            .push(MigrationRecoveryDiagnostic::error(
                MigrationDiagnosticCategory::Restore,
                message,
            ));
        self.refresh_legacy_settings(LegacyMigrationStatus::Failed);
    }

    fn start_empty_profile(&mut self) {
        let legacy_path = self.snapshot.migration_recovery.legacy_path.clone();
        self.snapshot.migration_recovery = MigrationRecoverySnapshot::default();
        self.snapshot.global_settings.legacy_migration.status = LegacyMigrationStatus::Skipped;
        self.snapshot.global_settings.legacy_migration.last_status =
            "Skipped; legacy data was left unchanged".to_owned();
        self.snapshot
            .global_settings
            .legacy_migration
            .legacy_path_hint = legacy_path;
    }

    fn open_connections_after_migration(&mut self) {
        self.snapshot.migration_recovery = MigrationRecoverySnapshot::default();
        self.open_default_connection_surface();
    }

    fn refresh_legacy_settings(&mut self, status: LegacyMigrationStatus) {
        let recovery = &self.snapshot.migration_recovery;
        let settings = &mut self.snapshot.global_settings.legacy_migration;
        settings.status = status;
        settings.last_status = status.label().to_owned();
        settings.legacy_path_hint = recovery.legacy_path.clone();
        settings.backup_name = recovery.backup_name.clone();
        settings.backup_path_hint = recovery.backup_path_hint.clone();
        settings.diagnostics_available = !recovery.diagnostics.is_empty();
        settings.restore_available = recovery.backup_name.is_some();
        settings.warning_count = recovery.warning_count();
    }
}
