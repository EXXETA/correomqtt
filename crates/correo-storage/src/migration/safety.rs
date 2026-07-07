use crate::current::{AppConfig, BuiltInBrokerConfig, ConfigStore, HistoryStore, ScriptStore};
use crate::{Result, StorageError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::MigrationPreview;

#[path = "safety/diagnostics.rs"]
mod diagnostics;

pub use diagnostics::{MappedLegacyField, MigrationDiagnostic, MigrationDiagnostics};

const BACKUP_MANIFEST_FILE: &str = ".correo-migration-backup.json";
const DIAGNOSTICS_FILE: &str = "migration-diagnostics.json";
const ROLLBACK_MARKER_FILE: &str = ".correo-migration-rollback.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationBackup {
    pub id: String,
    pub path: PathBuf,
    pub target_root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationApplyOutcome {
    pub backup: MigrationBackup,
    pub diagnostics: MigrationDiagnostics,
}

#[derive(Clone, Debug)]
pub struct MigrationApplier {
    target_root: PathBuf,
    backup_root: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BackupManifest {
    backup_id: String,
    source_existed: bool,
    target_root: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RollbackMarker {
    backup_id: String,
    backup_path: PathBuf,
    state_fingerprint: Option<String>,
}

impl MigrationApplier {
    pub fn new(target_root: impl Into<PathBuf>) -> Self {
        let target_root = target_root.into();
        let backup_root = default_backup_root(&target_root);
        Self {
            target_root,
            backup_root,
        }
    }

    pub fn with_backup_root(
        target_root: impl Into<PathBuf>,
        backup_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            target_root: target_root.into(),
            backup_root: backup_root.into(),
        }
    }

    pub fn backup_from_path(
        &self,
        id: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> MigrationBackup {
        MigrationBackup {
            id: id.into(),
            path: path.into(),
            target_root: self.target_root.clone(),
        }
    }

    pub fn apply_preview(&self, preview: &MigrationPreview) -> Result<MigrationApplyOutcome> {
        let backup = self.create_backup()?;
        let diagnostics = self.apply_preview_with_backup(preview, &backup)?;
        Ok(MigrationApplyOutcome {
            backup,
            diagnostics,
        })
    }

    pub fn apply_preview_with_backup(
        &self,
        preview: &MigrationPreview,
        backup: &MigrationBackup,
    ) -> Result<MigrationDiagnostics> {
        match self.apply_with_backup(preview, backup) {
            Ok(diagnostics) => Ok(diagnostics),
            Err(error) => {
                self.restore_backup_contents(backup)?;
                Err(error)
            }
        }
    }

    pub fn rollback(&self, backup: &MigrationBackup) -> Result<MigrationDiagnostics> {
        let marker = self.read_rollback_marker()?;
        if marker.backup_id != backup.id || marker.backup_path != backup.path {
            return Err(StorageError::MigrationRollbackSafety {
                reason: "rollback marker does not match the selected backup".to_owned(),
            });
        }
        if let Some(expected) = marker.state_fingerprint {
            let actual = directory_fingerprint(&self.target_root)?;
            if actual != expected {
                return Err(StorageError::MigrationRollbackSafety {
                    reason: "target data changed after migration; refusing to overwrite newer data"
                        .to_owned(),
                });
            }
        }
        self.restore_backup_contents(backup)?;
        Ok(MigrationDiagnostics::rollback_complete(backup))
    }

    pub fn accept_migration(&self, backup: &MigrationBackup) -> Result<()> {
        let marker = self.read_rollback_marker()?;
        if marker.backup_id != backup.id || marker.backup_path != backup.path {
            return Err(StorageError::MigrationRollbackSafety {
                reason: "rollback marker does not match the accepted backup".to_owned(),
            });
        }
        fs::remove_file(self.rollback_marker_path()).map_err(|source| StorageError::Delete {
            path: self.rollback_marker_path(),
            source,
        })
    }

    fn apply_with_backup(
        &self,
        preview: &MigrationPreview,
        backup: &MigrationBackup,
    ) -> Result<MigrationDiagnostics> {
        self.write_rollback_marker(&RollbackMarker {
            backup_id: backup.id.clone(),
            backup_path: backup.path.clone(),
            state_fingerprint: None,
        })?;
        self.write_preview(preview)?;
        let diagnostics = MigrationDiagnostics::from_preview(preview, Some(backup));
        write_json(&self.target_root.join(DIAGNOSTICS_FILE), &diagnostics)?;
        let fingerprint = directory_fingerprint(&self.target_root)?;
        self.write_rollback_marker(&RollbackMarker {
            backup_id: backup.id.clone(),
            backup_path: backup.path.clone(),
            state_fingerprint: Some(fingerprint),
        })?;
        Ok(diagnostics)
    }

    fn write_preview(&self, preview: &MigrationPreview) -> Result<()> {
        ConfigStore::new(&self.target_root).save(&AppConfig {
            connections: preview.connections.clone(),
            theme_settings: preview.theme_settings.clone(),
            settings: preview.settings.clone(),
            built_in_broker: BuiltInBrokerConfig::default(),
        })?;
        HistoryStore::new(&self.target_root).replace_all(&preview.histories)?;
        ScriptStore::new(&self.target_root).replace_all(&preview.scripts)
    }

    pub fn create_backup(&self) -> Result<MigrationBackup> {
        let id = timestamped_backup_id();
        let backup_path = self.backup_root.join(&id);
        ensure_copy_target_outside_source(&self.target_root, &backup_path)?;
        fs::create_dir_all(&self.backup_root).map_err(|source| StorageError::CreateDir {
            path: self.backup_root.clone(),
            source,
        })?;
        fs::create_dir_all(&backup_path).map_err(|source| StorageError::CreateDir {
            path: backup_path.clone(),
            source,
        })?;
        let source_existed = self.target_root.exists();
        if source_existed {
            copy_dir_contents(&self.target_root, &backup_path, &[ROLLBACK_MARKER_FILE])?;
        }
        write_json(
            &backup_path.join(BACKUP_MANIFEST_FILE),
            &BackupManifest {
                backup_id: id.clone(),
                source_existed,
                target_root: self.target_root.clone(),
            },
        )?;
        Ok(MigrationBackup {
            id,
            path: backup_path,
            target_root: self.target_root.clone(),
        })
    }

    fn restore_backup_contents(&self, backup: &MigrationBackup) -> Result<()> {
        let manifest: BackupManifest = read_json(backup.path.join(BACKUP_MANIFEST_FILE))?;
        if manifest.backup_id != backup.id || manifest.target_root != self.target_root {
            return Err(StorageError::MigrationRollbackSafety {
                reason: "backup manifest does not match the migration target".to_owned(),
            });
        }
        remove_path_if_exists(&self.target_root)?;
        if !manifest.source_existed {
            return Ok(());
        }
        fs::create_dir_all(&self.target_root).map_err(|source| StorageError::CreateDir {
            path: self.target_root.clone(),
            source,
        })?;
        copy_dir_contents(&backup.path, &self.target_root, &[BACKUP_MANIFEST_FILE])
    }

    fn read_rollback_marker(&self) -> Result<RollbackMarker> {
        read_json(self.rollback_marker_path())
    }

    fn write_rollback_marker(&self, marker: &RollbackMarker) -> Result<()> {
        write_json(&self.rollback_marker_path(), marker)
    }

    fn rollback_marker_path(&self) -> PathBuf {
        self.target_root.join(ROLLBACK_MARKER_FILE)
    }
}

fn timestamped_backup_id() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "migration-backup-{}-{:09}",
        elapsed.as_secs(),
        elapsed.subsec_nanos()
    )
}

fn default_backup_root(target_root: &Path) -> PathBuf {
    let name = target_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("correomqtt-data");
    target_root.with_file_name(format!("{name}-migration-backups"))
}

fn copy_dir_contents(source: &Path, target: &Path, skip_names: &[&str]) -> Result<()> {
    ensure_copy_target_outside_source(source, target)?;
    fs::create_dir_all(target).map_err(|source_error| StorageError::CreateDir {
        path: target.to_path_buf(),
        source: source_error,
    })?;
    for entry in fs::read_dir(source).map_err(|source_error| StorageError::Read {
        path: source.to_path_buf(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| StorageError::Read {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let path = entry.path();
        let Some(name) = path.file_name() else {
            continue;
        };
        if skip_names.iter().any(|skip| name == *skip) {
            continue;
        }
        let target_path = target.join(name);
        if path.is_dir() {
            copy_dir_contents(&path, &target_path, skip_names)?;
        } else {
            fs::copy(&path, &target_path).map_err(|source_error| StorageError::Copy {
                from: path,
                to: target_path,
                source: source_error,
            })?;
        }
    }
    Ok(())
}

fn ensure_copy_target_outside_source(source: &Path, target: &Path) -> Result<()> {
    let source = normalize_path(source);
    let target = normalize_path(target);
    if target == source || target.starts_with(&source) {
        return Err(StorageError::MigrationRollbackSafety {
            reason: "migration backup destination must be outside the migration target".to_owned(),
        });
    }
    Ok(())
}

fn normalize_path(path: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn directory_fingerprint(root: &Path) -> Result<String> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort();
    let mut hasher = Sha256::new();
    for relative_path in files {
        if relative_path == Path::new(ROLLBACK_MARKER_FILE) {
            continue;
        }
        let absolute_path = root.join(&relative_path);
        hasher.update(relative_path.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update(
            fs::read(&absolute_path).map_err(|source| StorageError::Read {
                path: absolute_path,
                source,
            })?,
        );
        hasher.update([0]);
    }
    Ok(hex_digest(hasher.finalize().as_slice()))
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|source| StorageError::Read {
        path: dir.to_path_buf(),
        source,
    })? {
        let path = entry
            .map_err(|source| StorageError::Read {
                path: dir.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else {
            out.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|source| StorageError::Delete {
            path: path.to_path_buf(),
            source,
        })
    } else {
        fs::remove_file(path).map_err(|source| StorageError::Delete {
            path: path.to_path_buf(),
            source,
        })
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: impl Into<PathBuf>) -> Result<T> {
    let path = path.into();
    let text = fs::read_to_string(&path).map_err(|source| StorageError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| StorageError::Json { path, source })
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| StorageError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(value).map_err(|source| StorageError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    write_file_atomic(path, json.as_bytes())
}

fn write_file_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&temporary, content).map_err(|source| StorageError::Write {
        path: temporary.clone(),
        source,
    })?;
    if cfg!(windows) && path.exists() {
        fs::remove_file(path).map_err(|source| StorageError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    }
    fs::rename(&temporary, path).map_err(|source| StorageError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
