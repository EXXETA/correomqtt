use super::*;

impl ScriptStore {
    pub(super) fn replacement_root(&self, purpose: &str) -> PathBuf {
        self.scripts_root.with_file_name(format!(
            ".scripts-{purpose}-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ))
    }

    fn recovery_marker_path(&self) -> PathBuf {
        self.scripts_root.with_file_name(".scripts-recovery")
    }

    fn write_recovery_marker(&self, backup_root: &Path) -> Result<()> {
        let marker = self.recovery_marker_path();
        ensure_parent_dir(&marker)?;
        write_text(&marker, &backup_root.to_string_lossy())?;
        sync_parent_dir(&marker)
    }

    fn clear_recovery_marker(&self) -> Result<()> {
        let marker = self.recovery_marker_path();
        remove_file_if_exists(marker.clone())?;
        sync_parent_dir(&marker)
    }

    pub(super) fn commit_staging(&self, staging_root: PathBuf) -> Result<()> {
        let live_root = self.scripts_root();
        let backup_root = self.replacement_root("backup");
        if live_root.exists() {
            self.write_recovery_marker(&backup_root)?;
            self.rename_for_commit(&live_root, &backup_root)?;
        }
        if let Err(error) = self.rename_for_commit(&staging_root, &live_root) {
            if backup_root.exists() {
                if let Err(recovery_error) = self.rename_for_commit(&backup_root, &live_root) {
                    return Err(Self::recovery_error(backup_root, recovery_error));
                }
            }
            self.clear_recovery_marker()?;
            return Err(error);
        }
        self.clear_recovery_marker()?;
        let _ = remove_dir_if_exists(backup_root);
        Ok(())
    }

    pub(super) fn recover_live_root(&self) -> Result<()> {
        let marker = self.recovery_marker_path();
        if !marker.exists() {
            return Ok(());
        }

        let backup_root = self.read_recovery_marker(&marker)?;
        let live_root = self.scripts_root();
        if live_root.exists() {
            self.clear_recovery_marker()?;
            let _ = remove_dir_if_exists(backup_root);
            return Ok(());
        }
        if !backup_root.exists() {
            return Err(Self::recovery_error(
                backup_root,
                StorageError::Read {
                    path: marker,
                    source: std::io::Error::other("recovery marker backup is missing"),
                },
            ));
        }
        self.rename_for_commit(&backup_root, &live_root)
            .map_err(|error| Self::recovery_error(backup_root.clone(), error))?;
        self.clear_recovery_marker()
    }

    fn read_recovery_marker(&self, marker: &Path) -> Result<PathBuf> {
        let backup_root = PathBuf::from(crate::error::read_to_string(marker)?.trim());
        let is_backup = backup_root.parent() == self.scripts_root.parent()
            && backup_root
                .file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.starts_with(".scripts-backup-"));
        if is_backup {
            return Ok(backup_root);
        }
        Err(StorageError::Read {
            path: marker.to_path_buf(),
            source: std::io::Error::other(
                "invalid scripts recovery marker; inspect and restore the previous scripts backup",
            ),
        })
    }

    fn recovery_error(backup_root: PathBuf, recovery_error: StorageError) -> StorageError {
        StorageError::Read {
            path: backup_root.clone(),
            source: std::io::Error::other(format!(
                "could not restore previous scripts tree; backup retained at {}: {recovery_error}",
                backup_root.display()
            )),
        }
    }

    fn rename_for_commit(&self, source: &Path, destination: &Path) -> Result<()> {
        #[cfg(test)]
        {
            let mut failures = self.failures.lock().expect("failure injection lock");
            failures.rename_attempts += 1;
            let attempt = failures.rename_attempts;
            if failures.rename_failure_attempts.remove(&attempt) {
                return rename_path(source, &destination.join(".injected-rename-failure"));
            }
        }
        rename_path(source, destination)?;
        sync_parent_dir(destination)
    }

    pub(super) fn write_script_text(&self, path: &Path, source: &str) -> Result<()> {
        #[cfg(test)]
        {
            let mut failures = self.failures.lock().expect("failure injection lock");
            if let Some(remaining) = failures.staged_write_failure_after {
                if remaining == 0 {
                    failures.staged_write_failure_after = None;
                    return Err(StorageError::Write {
                        path: path.to_path_buf(),
                        source: std::io::Error::other("injected staged write failure"),
                    });
                }
                failures.staged_write_failure_after = Some(remaining - 1);
            }
        }
        write_text(path, source)
    }

    #[cfg(test)]
    fn fail_staged_write_after(&self, successful_writes: usize) {
        self.failures
            .lock()
            .expect("failure injection lock")
            .staged_write_failure_after = Some(successful_writes);
    }

    #[cfg(test)]
    fn fail_rename_attempts(&self, attempts: &[usize]) {
        let mut failures = self.failures.lock().expect("failure injection lock");
        failures.rename_attempts = 0;
        failures
            .rename_failure_attempts
            .extend(attempts.iter().copied());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn execution(script_path: &str) -> ScriptExecution {
        ScriptExecution {
            execution_id: "kept-execution".to_owned(),
            script_name: "kept.js".to_owned(),
            script_path: script_path.into(),
            connection_id: Some("connection".to_owned()),
            status: ScriptExecutionStatus::Succeeded,
            error: None,
            started_at: Some("2026-01-01T00:00:00.000".to_owned()),
            ended_at: Some("2026-01-01T00:00:01.000".to_owned()),
            duration_ms: Some(1_000),
            cancelled: false,
            log_path: None,
        }
    }

    #[test]
    fn replace_all_keeps_the_complete_previous_tree_when_a_staged_write_fails() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = ScriptStore::new(temp.path());
        let kept_path = "kept.js";
        store
            .create_script(kept_path, "console.log('kept');")
            .expect("create previous script");
        let kept_execution = execution(kept_path);
        store
            .save_execution(kept_path, &kept_execution)
            .expect("save previous execution");
        store
            .append_log_record(
                kept_path,
                &ScriptLogRecord {
                    execution_id: kept_execution.execution_id.clone(),
                    sequence: 0,
                    timestamp: None,
                    level: ScriptLogLevel::Info,
                    message: "kept log".to_owned(),
                },
            )
            .expect("write previous log");
        let previous = store.load_snapshot(10).expect("load previous tree");
        let replacement = ScriptPersistenceSnapshot {
            files: vec![
                ScriptFile::new("first.js".into(), "console.log('first');".to_owned()),
                ScriptFile::new("second.js".into(), "console.log('second');".to_owned()),
            ],
            ..ScriptPersistenceSnapshot::default()
        };

        store.fail_staged_write_after(1);

        assert!(matches!(
            store.replace_all(&replacement),
            Err(StorageError::Write { .. })
        ));
        assert_eq!(
            store.load_snapshot(10).expect("load preserved tree"),
            previous
        );

        let mut root_entries = std::fs::read_dir(temp.path())
            .expect("read data root")
            .map(|entry| entry.expect("directory entry").file_name())
            .collect::<Vec<_>>();
        root_entries.sort();
        assert_eq!(root_entries, vec![std::ffi::OsString::from("scripts")]);
    }

    #[test]
    fn replace_all_restores_previous_tree_when_staging_commit_rename_fails() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = ScriptStore::new(temp.path());
        store
            .create_script("previous.js", "console.log('previous');")
            .expect("create previous script");
        let previous = store.load_snapshot(10).expect("load previous tree");
        let replacement = ScriptPersistenceSnapshot {
            files: vec![ScriptFile::new(
                "replacement.js".into(),
                "console.log('replacement');".to_owned(),
            )],
            ..ScriptPersistenceSnapshot::default()
        };

        store.fail_rename_attempts(&[2]);

        assert!(store.replace_all(&replacement).is_err());
        assert_eq!(
            store.load_snapshot(10).expect("load restored tree"),
            previous
        );
        assert_eq!(
            std::fs::read_dir(temp.path())
                .expect("read data root")
                .map(|entry| entry.expect("directory entry").file_name())
                .collect::<Vec<_>>(),
            vec![std::ffi::OsString::from("scripts")]
        );
    }

    #[test]
    fn replace_all_recovers_previous_tree_from_durable_backup_after_restart() {
        let temp = tempfile::tempdir().expect("tempdir");
        fn assert_sync<T: Sync>() {}

        assert_sync::<ScriptStore>();
        let store = ScriptStore::new(temp.path());
        store
            .create_script("previous.js", "console.log('previous');")
            .expect("create previous script");
        let previous = store.load_snapshot(10).expect("load previous tree");
        let replacement = ScriptPersistenceSnapshot {
            files: vec![ScriptFile::new(
                "replacement.js".into(),
                "console.log('replacement');".to_owned(),
            )],
            ..ScriptPersistenceSnapshot::default()
        };

        store.fail_rename_attempts(&[2, 3]);

        let error = store
            .replace_all(&replacement)
            .expect_err("commit must fail");
        assert!(!temp.path().join("scripts").exists());
        let marker = temp.path().join(".scripts-recovery");
        assert!(marker.exists());
        assert!(error.to_string().contains(".scripts-backup-"));
        assert_eq!(
            store
                .load_script("previous.js")
                .expect("recover previous script before direct access"),
            previous.files[0]
        );
        assert!(!marker.exists());

        let fresh_store = ScriptStore::new(temp.path());
        assert_eq!(
            fresh_store
                .load_snapshot(10)
                .expect("recover previous tree after restart"),
            previous
        );
        assert_eq!(
            std::fs::read_dir(temp.path())
                .expect("read data root")
                .map(|entry| entry.expect("directory entry").file_name())
                .collect::<Vec<_>>(),
            vec![std::ffi::OsString::from("scripts")]
        );
    }
}
