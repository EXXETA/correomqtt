use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use correo_diagnostics::redact_sensitive;
use correo_storage::current::{
    default_secret_store, ImportedSecret, SecretMaterial, SecretReference, SecretStore,
};
use correo_storage::legacy::{
    passwords::{os_keyring_master_password, LegacyPasswords},
    LegacyProfile,
};
use correo_storage::migration::{
    configured_secret_slots, imported_connection_secrets, MigrationApplier, MigrationBackup,
    MigrationDiagnostics, MigrationPreview, MigrationWarning,
};
use correo_storage::StorageError;
use thiserror::Error;

use crate::{
    startup_state_from_migration, AppEvent, MigrationApplyStage, MigrationDiagnosticCategory,
    MigrationFailureStage, MigrationRecoveryCompletion, MigrationRecoveryCounts,
    MigrationRecoveryDiagnostic, MigrationRecoveryEvent, MigrationRecoveryFailure,
    MigrationRecoveryRow, MigrationRecoverySnapshot, MigrationRecoveryTask,
    MigrationRecoveryWarning, MigrationRecoveryWarningKind, SecretInput, ThemeMode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationPersistenceCommand {
    Prepare {
        legacy_path: String,
    },
    UnlockSecrets {
        master_password: SecretInput,
    },
    SkipSecrets,
    Apply {
        fallback_theme: ThemeMode,
    },
    Restore {
        backup_name: String,
        backup_path_hint: String,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MigrationDispatchError {
    #[error("migration persistence worker is stopped")]
    Stopped,
}

#[derive(Debug)]
pub struct MigrationPersistenceWorker {
    sender: Sender<MigrationPersistenceCommand>,
    events: Receiver<AppEvent>,
}

#[derive(Debug)]
struct PendingMigration {
    legacy_path: PathBuf,
    preview: MigrationPreview,
    backup: MigrationBackup,
    imported_secrets: Vec<ImportedSecret>,
    skipped_secret_count: usize,
}

#[derive(Debug, Clone)]
struct RecoverableBackup {
    backup: MigrationBackup,
}

impl MigrationPersistenceWorker {
    pub fn start(current_root: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let applier = MigrationApplier::new(current_root);

        std::thread::spawn(move || {
            let mut pending = None;
            let mut recoverable_backup = None;
            while let Ok(command) = receiver.recv() {
                for event in
                    handle_command(&applier, &mut pending, &mut recoverable_backup, command)
                {
                    let _ = events_sender.send(event);
                }
            }
        });

        Self { sender, events }
    }

    pub fn dispatch(
        &self,
        command: MigrationPersistenceCommand,
    ) -> Result<(), MigrationDispatchError> {
        self.sender
            .send(command)
            .map_err(|_| MigrationDispatchError::Stopped)
    }

    pub fn try_recv_event(&self) -> Option<AppEvent> {
        self.events.try_recv().ok()
    }

    pub fn recv_event_timeout(&self, timeout: Duration) -> Option<AppEvent> {
        self.events.recv_timeout(timeout).ok()
    }
}

fn handle_command(
    applier: &MigrationApplier,
    pending: &mut Option<PendingMigration>,
    recoverable_backup: &mut Option<RecoverableBackup>,
    command: MigrationPersistenceCommand,
) -> Vec<AppEvent> {
    match command {
        MigrationPersistenceCommand::Prepare { legacy_path } => {
            prepare(applier, pending, legacy_path)
        }
        MigrationPersistenceCommand::UnlockSecrets { master_password } => {
            unlock_secrets(pending.as_mut(), &master_password)
        }
        MigrationPersistenceCommand::SkipSecrets => skip_secrets(pending.as_mut()),
        MigrationPersistenceCommand::Apply { fallback_theme } => {
            apply(applier, pending.take(), recoverable_backup, fallback_theme)
        }
        MigrationPersistenceCommand::Restore {
            backup_name,
            backup_path_hint,
        } => restore(
            applier,
            recoverable_backup.as_ref(),
            backup_name,
            backup_path_hint,
        ),
    }
}

fn prepare(
    applier: &MigrationApplier,
    pending: &mut Option<PendingMigration>,
    legacy_path: String,
) -> Vec<AppEvent> {
    match prepare_pending(applier, legacy_path) {
        Ok(prepared) => {
            let backup_name = prepared.backup.id.clone();
            let backup_path_hint = prepared.backup.path.display().to_string();
            *pending = Some(prepared);
            let mut events = vec![migration_event(MigrationRecoveryEvent::BackupCreated {
                backup_name,
                backup_path_hint,
            })];
            events.extend(auto_unlock_or_prompt(pending.as_mut()));
            events
        }
        Err(message) => vec![migration_event(MigrationRecoveryEvent::BackupFailed {
            message,
        })],
    }
}

fn prepare_pending(
    applier: &MigrationApplier,
    legacy_path: String,
) -> Result<PendingMigration, String> {
    let legacy_path = PathBuf::from(legacy_path);
    let profile = LegacyProfile::read_from(&legacy_path).map_err(|error| {
        format!("Legacy profile could not be read before migration backup: {error}")
    })?;
    let preview = MigrationPreview::from_legacy_profile(profile)
        .map_err(|error| format!("Legacy profile could not be prepared for migration: {error}"))?;
    let backup = applier
        .create_backup()
        .map_err(|error| format!("Migration backup could not be created: {error}"))?;
    Ok(PendingMigration {
        legacy_path,
        preview,
        backup,
        imported_secrets: Vec::new(),
        skipped_secret_count: 0,
    })
}

fn review_ready(pending: Option<&PendingMigration>) -> Vec<AppEvent> {
    match pending {
        Some(pending) => vec![migration_event(MigrationRecoveryEvent::ReviewReady {
            counts: preview_counts(pending),
            rows: review_rows(&pending.preview),
            warnings: preview_warnings(&pending.preview.warnings),
        })],
        None => vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
            failure: MigrationRecoveryFailure {
                stage: MigrationFailureStage::BeforeWrite,
                message: "Migration preview is not prepared; start migration again.".to_owned(),
            },
        })],
    }
}

fn auto_unlock_or_prompt(pending: Option<&mut PendingMigration>) -> Vec<AppEvent> {
    if let Some(pending) = pending {
        if let Some(password) = os_keyring_master_password() {
            if let Ok(imported_count) = import_legacy_secrets(pending, &password) {
                return secrets_unlocked_events(pending, imported_count);
            }
        }
    }
    vec![migration_event(MigrationRecoveryEvent::PasswordNeeded)]
}

fn secrets_unlocked_events(pending: &mut PendingMigration, imported_count: usize) -> Vec<AppEvent> {
    pending.skipped_secret_count = 0;
    let mut events = vec![migration_event(MigrationRecoveryEvent::SecretsUnlocked {
        imported_count,
    })];
    events.extend(review_ready(Some(pending)));
    events
}

fn unlock_secrets(
    pending: Option<&mut PendingMigration>,
    master_password: &SecretInput,
) -> Vec<AppEvent> {
    let Some(pending) = pending else {
        return preview_not_prepared();
    };
    let password = master_password.expose_for_ui();
    if password.trim().is_empty() {
        return vec![migration_event(MigrationRecoveryEvent::PasswordRejected)];
    }

    match import_legacy_secrets(pending, password) {
        Ok(imported_count) => secrets_unlocked_events(pending, imported_count),
        Err(UnlockSecretsError::WrongPassword) => {
            vec![migration_event(MigrationRecoveryEvent::PasswordRejected)]
        }
        Err(UnlockSecretsError::UnsupportedEncryption) => {
            vec![migration_event(
                MigrationRecoveryEvent::UnsupportedEncryption,
            )]
        }
        Err(UnlockSecretsError::Failed(message)) => {
            vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
                failure: MigrationRecoveryFailure {
                    stage: MigrationFailureStage::BeforeWrite,
                    message,
                },
            })]
        }
    }
}

fn skip_secrets(pending: Option<&mut PendingMigration>) -> Vec<AppEvent> {
    let Some(pending) = pending else {
        return preview_not_prepared();
    };
    pending.skipped_secret_count = configured_secret_slots(&pending.preview.connections);
    let mut events = vec![migration_event(MigrationRecoveryEvent::SecretsSkipped {
        skipped_count: pending.skipped_secret_count,
    })];
    events.extend(review_ready(Some(pending)));
    events
}

fn preview_not_prepared() -> Vec<AppEvent> {
    vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
        failure: MigrationRecoveryFailure {
            stage: MigrationFailureStage::BeforeWrite,
            message: "Migration preview is not prepared; start migration again.".to_owned(),
        },
    })]
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UnlockSecretsError {
    WrongPassword,
    UnsupportedEncryption,
    Failed(String),
}

fn import_legacy_secrets(
    pending: &mut PendingMigration,
    master_password: &str,
) -> Result<usize, UnlockSecretsError> {
    let path = pending.legacy_path.join("passwords.json");
    if !path.exists() {
        return Ok(0);
    }
    let passwords = LegacyPasswords::read_from(&path).map_err(unlock_error)?;
    let decrypted = passwords.decrypt(master_password).map_err(unlock_error)?;
    let secrets = imported_connection_secrets(&decrypted, &pending.preview.connections);
    let imported_count = secrets.len();
    pending.imported_secrets = secrets;
    Ok(imported_count)
}

fn apply(
    applier: &MigrationApplier,
    pending: Option<PendingMigration>,
    recoverable_backup: &mut Option<RecoverableBackup>,
    fallback_theme: ThemeMode,
) -> Vec<AppEvent> {
    let Some(pending) = pending else {
        return vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
            failure: MigrationRecoveryFailure {
                stage: MigrationFailureStage::BeforeWrite,
                message: "Migration preview is not prepared; start migration again.".to_owned(),
            },
        })];
    };
    let backup = pending.backup.clone();
    let secret_store = default_secret_store();
    let secret_snapshot = match snapshot_secrets(secret_store.as_ref(), &pending.imported_secrets) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
                failure: MigrationRecoveryFailure {
                    stage: MigrationFailureStage::BeforeWrite,
                    message: format!("Migration secrets could not be snapshotted: {error}"),
                },
            })];
        }
    };
    if let Err(error) = secret_store.put_all(&pending.imported_secrets) {
        let _ = restore_secret_snapshot(secret_store.as_ref(), secret_snapshot.clone());
        return vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
            failure: MigrationRecoveryFailure {
                stage: MigrationFailureStage::BeforeWrite,
                message: format!("Migration secrets could not be written: {error}"),
            },
        })];
    }
    match applier.apply_preview_with_backup(&pending.preview, &backup) {
        Ok(diagnostics) => {
            *recoverable_backup = Some(RecoverableBackup { backup });
            let completion = completion_from_diagnostics(&diagnostics);
            let state = startup_state_from_migration(pending.preview, fallback_theme);
            vec![
                migration_event(MigrationRecoveryEvent::ApplyStageChanged {
                    stage: MigrationApplyStage::ConfigWritten,
                }),
                AppEvent::MigrationApplied {
                    state: Box::new(state),
                    completion,
                    diagnostics: recovery_diagnostics(&diagnostics),
                },
            ]
        }
        Err(error) => {
            let _ = restore_secret_snapshot(secret_store.as_ref(), secret_snapshot);
            *recoverable_backup = Some(RecoverableBackup { backup });
            vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
                failure: MigrationRecoveryFailure {
                    stage: MigrationFailureStage::AfterWrite,
                    message: format!("Migration apply failed after backup: {error}"),
                },
            })]
        }
    }
}

fn restore(
    applier: &MigrationApplier,
    backup: Option<&RecoverableBackup>,
    backup_name: String,
    backup_path_hint: String,
) -> Vec<AppEvent> {
    let backup = backup
        .map(|backup| backup.backup.clone())
        .unwrap_or_else(|| applier.backup_from_path(backup_name, backup_path_hint));
    match applier.rollback(&backup) {
        Ok(_) => vec![migration_event(MigrationRecoveryEvent::RestoreCompleted)],
        Err(error) => vec![migration_event(MigrationRecoveryEvent::RestoreFailed {
            message: error.to_string(),
        })],
    }
}

fn snapshot_secrets(
    store: &dyn SecretStore,
    secrets: &[ImportedSecret],
) -> Result<Vec<(SecretReference, Option<SecretMaterial>)>, StorageError> {
    let references = secrets
        .iter()
        .map(|secret| secret.reference.clone())
        .collect::<Vec<_>>();
    let values = store.get_all(&references)?;
    Ok(references.into_iter().zip(values).collect())
}

fn restore_secret_snapshot(
    store: &dyn SecretStore,
    snapshot: Vec<(SecretReference, Option<SecretMaterial>)>,
) -> Result<(), StorageError> {
    let mut puts = Vec::new();
    let mut deletes = Vec::new();
    for (reference, value) in snapshot {
        if let Some(value) = value {
            puts.push(ImportedSecret { reference, value });
        } else {
            deletes.push(reference);
        }
    }
    store.apply(&puts, &deletes)
}

fn migration_event(event: MigrationRecoveryEvent) -> AppEvent {
    AppEvent::MigrationRecovery(event)
}

fn preview_counts(pending: &PendingMigration) -> MigrationRecoveryCounts {
    MigrationRecoveryCounts {
        connections: pending.preview.connections.len(),
        histories: pending.preview.histories.connections.len(),
        scripts: pending.preview.scripts.files.len(),
        plugin_artifacts_ignored: pending.preview.plugin_state.ignored_legacy_paths.len(),
        warnings: pending.preview.warnings.len(),
        skipped_secrets: pending.skipped_secret_count,
    }
}

fn unlock_error(error: StorageError) -> UnlockSecretsError {
    match error {
        StorageError::PasswordDecryption => UnlockSecretsError::WrongPassword,
        StorageError::UnsupportedPasswordEncryption(_)
        | StorageError::InvalidPasswordPayload(_) => UnlockSecretsError::UnsupportedEncryption,
        other => UnlockSecretsError::Failed(other.to_string()),
    }
}

