use crate::{Result, StorageError};
use std::fs;
use std::path::{Path, PathBuf};

use super::remove_path_if_exists;

const ROLLBACK_PREVIOUS: &str = "rollback-previous";
const ROLLBACK_STAGING: &str = "rollback-staging";

pub(super) fn replace_target(target: &Path, staged_restore: Option<&Path>) -> Result<()> {
    let displaced_target = if target.exists() {
        let path = replacement_path(target, ROLLBACK_PREVIOUS);
        if let Err(source) = fs::rename(target, &path) {
            if let Some(staged_restore) = staged_restore {
                let _ = remove_path_if_exists(staged_restore);
            }
            return Err(StorageError::Rename {
                from: target.to_path_buf(),
                to: path,
                source,
            });
        }
        Some(path)
    } else {
        None
    };

    if let Some(staged_restore) = staged_restore {
        if let Err(source) = fs::rename(staged_restore, target) {
            if let Some(displaced_target) = &displaced_target {
                if let Err(restore_error) = fs::rename(displaced_target, target) {
                    return Err(StorageError::MigrationRollbackSafety {
                        reason: format!(
                            "staged rollback could not replace the target ({source}); the previous target could not be restored ({restore_error})"
                        ),
                    });
                }
            }
            let _ = remove_path_if_exists(staged_restore);
            return Err(StorageError::Rename {
                from: staged_restore.to_path_buf(),
                to: target.to_path_buf(),
                source,
            });
        }
    }

    // The swap is done; a leftover copy must not turn a completed rollback into a failure.
    if let Some(displaced_target) = displaced_target {
        let _ = remove_path_if_exists(&displaced_target);
    }
    Ok(())
}

pub(super) fn replacement_path(target: &Path, purpose: &str) -> PathBuf {
    target.with_file_name(format!(
        "{}{}-{}",
        sibling_prefix(target, purpose),
        std::process::id(),
        rand::random::<u64>()
    ))
}

/// Puts the data root back when a rollback stopped between its two renames.
///
/// Returns `true` when a displaced data root was moved back into place.
pub fn recover_interrupted_rollback(target_root: &Path) -> Result<bool> {
    if target_root.exists() {
        return Ok(false);
    }
    let mut displaced = siblings_with_prefix(target_root, ROLLBACK_PREVIOUS)?;
    let Some(previous) = displaced.pop() else {
        return Ok(false);
    };
    if !displaced.is_empty() {
        return Err(StorageError::MigrationRollbackSafety {
            reason: format!(
                "more than one interrupted rollback was found next to {}; restore one manually",
                target_root.display()
            ),
        });
    }
    fs::rename(&previous, target_root).map_err(|source| StorageError::Rename {
        from: previous.clone(),
        to: target_root.to_path_buf(),
        source,
    })?;
    for staging in siblings_with_prefix(target_root, ROLLBACK_STAGING)? {
        let _ = remove_path_if_exists(&staging);
    }
    Ok(true)
}

fn sibling_prefix(target: &Path, purpose: &str) -> String {
    let target_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("correomqtt-data");
    format!(".{target_name}-{purpose}-")
}

fn siblings_with_prefix(target: &Path, purpose: &str) -> Result<Vec<PathBuf>> {
    let Some(parent) = target.parent().filter(|parent| parent.is_dir()) else {
        return Ok(Vec::new());
    };
    let prefix = sibling_prefix(target, purpose);
    let read_error = |source| StorageError::Read {
        path: parent.to_path_buf(),
        source,
    };
    let mut siblings = Vec::new();
    for entry in fs::read_dir(parent).map_err(read_error)? {
        let entry = entry.map_err(read_error)?;
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            siblings.push(entry.path());
        }
    }
    Ok(siblings)
}
