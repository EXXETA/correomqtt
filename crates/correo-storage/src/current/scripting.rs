use crate::{Result, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

#[path = "scripting_helpers.rs"]
mod helpers;

use helpers::{
    collect_script_paths, display_path, ensure_dir, ensure_parent_dir, normalize_script_path,
    remove_dir_if_exists, remove_file_if_exists, rename_path, sync_parent_dir, write_text,
};
#[path = "scripting_log.rs"]
mod log;
#[path = "scripting_replace.rs"]
mod replace;

pub use helpers::redact_script_log_text;
pub use log::{BoundedScriptLog, ScriptLogLevel, ScriptLogRecord};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptPersistenceSnapshot {
    pub files: Vec<ScriptFile>,
    pub executions: Vec<ScriptExecution>,
    pub logs: Vec<ScriptLogRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptFile {
    pub name: String,
    pub relative_path: PathBuf,
    pub source: String,
}

impl ScriptFile {
    pub fn new(relative_path: PathBuf, source: String) -> Self {
        let name = relative_path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_owned();
        Self {
            name,
            relative_path,
            source,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptDirtyState {
    pub relative_path: PathBuf,
    pub dirty: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptExecution {
    pub execution_id: String,
    pub script_name: String,
    pub script_path: PathBuf,
    pub connection_id: Option<String>,
    pub status: ScriptExecutionStatus,
    pub error: Option<ScriptExecutionError>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub cancelled: bool,
    pub log_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptExecutionStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptExecutionError {
    pub error_type: ScriptExecutionErrorType,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptExecutionErrorType {
    Guest,
    Host,
}

#[derive(Clone, Debug)]
pub struct ScriptStore {
    scripts_root: PathBuf,
    #[cfg(test)]
    failures: Arc<Mutex<FailureInjection>>,
}

#[cfg(test)]
#[derive(Debug, Default)]
struct FailureInjection {
    staged_write_failure_after: Option<usize>,
    rename_attempts: usize,
    rename_failure_attempts: BTreeSet<usize>,
}

impl ScriptStore {
    pub fn new(data_root: impl Into<PathBuf>) -> Self {
        Self::with_scripts_root(data_root.into().join("scripts"))
    }

    pub fn list_scripts(&self) -> Result<Vec<ScriptFile>> {
        self.recover_live_root()?;
        let root = self.scripts_root();
        ensure_dir(&root)?;
        let mut paths = Vec::new();
        collect_script_paths(&root, &root, &mut paths)?;
        paths.sort();
        paths
            .into_iter()
            .map(|path| self.load_script(path))
            .collect()
    }

    pub fn load_snapshot(&self, max_log_records: usize) -> Result<ScriptPersistenceSnapshot> {
        let files = self.list_scripts()?;
        let mut executions = Vec::new();
        let mut logs = Vec::new();
        for file in &files {
            let script_executions = self.load_executions(&file.relative_path)?;
            for execution in &script_executions {
                let loaded = self.load_log(
                    &file.relative_path,
                    &execution.execution_id,
                    max_log_records,
                )?;
                logs.extend(loaded.records);
            }
            executions.extend(script_executions);
        }
        Ok(ScriptPersistenceSnapshot {
            files,
            executions,
            logs,
        })
    }

    pub fn replace_all(&self, snapshot: &ScriptPersistenceSnapshot) -> Result<()> {
        self.recover_live_root()?;
        self.validate_snapshot(snapshot)?;

        let staging_root = self.replacement_root("staging");
        ensure_parent_dir(&staging_root)?;
        fs::create_dir(&staging_root).map_err(|source| StorageError::CreateDir {
            path: staging_root.clone(),
            source,
        })?;
        let staging_store = Self::with_scripts_root(staging_root.clone());
        #[cfg(test)]
        let staging_store = {
            let mut staging_store = staging_store;
            staging_store.failures = Arc::clone(&self.failures);
            staging_store
        };

        let staged_result = staging_store
            .write_snapshot(snapshot)
            .and_then(|()| staging_store.load_snapshot(usize::MAX).map(|_| ()));
        if let Err(error) = staged_result {
            let _ = remove_dir_if_exists(staging_root);
            return Err(error);
        }

        if let Err(error) = self.commit_staging(staging_root.clone()) {
            let _ = remove_dir_if_exists(staging_root);
            return Err(error);
        }
        Ok(())
    }

    pub fn create_script(&self, script_path: impl AsRef<Path>, source: &str) -> Result<ScriptFile> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        let absolute_path = self.script_absolute_path(&relative_path);
        if absolute_path.exists() {
            return Err(StorageError::ScriptAlreadyExists(display_path(
                &relative_path,
            )));
        }
        ensure_parent_dir(&absolute_path)?;
        self.write_script_text(&absolute_path, source)?;
        Ok(ScriptFile::new(relative_path, source.to_owned()))
    }

    pub fn load_script(&self, script_path: impl AsRef<Path>) -> Result<ScriptFile> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        let absolute_path = self.script_absolute_path(&relative_path);
        if !absolute_path.exists() {
            return Err(StorageError::ScriptNotFound(display_path(&relative_path)));
        }
        let source = crate::error::read_to_string(absolute_path)?;
        Ok(ScriptFile::new(relative_path, source))
    }

    pub fn update_script(&self, script_path: impl AsRef<Path>, source: &str) -> Result<ScriptFile> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        let absolute_path = self.script_absolute_path(&relative_path);
        if !absolute_path.exists() {
            return Err(StorageError::ScriptNotFound(display_path(&relative_path)));
        }
        write_text(&absolute_path, source)?;
        Ok(ScriptFile::new(relative_path, source.to_owned()))
    }

    pub fn rename_script(
        &self,
        old_path: impl AsRef<Path>,
        new_path: impl AsRef<Path>,
    ) -> Result<ScriptFile> {
        self.recover_live_root()?;
        let old_relative = normalize_script_path(old_path.as_ref())?;
        let new_relative = normalize_script_path(new_path.as_ref())?;
        if old_relative == new_relative {
            return Err(StorageError::ScriptNameUnchanged(display_path(
                &old_relative,
            )));
        }
        let old_absolute = self.script_absolute_path(&old_relative);
        let new_absolute = self.script_absolute_path(&new_relative);
        if !old_absolute.exists() {
            return Err(StorageError::ScriptNotFound(display_path(&old_relative)));
        }
        if new_absolute.exists() {
            return Err(StorageError::ScriptAlreadyExists(display_path(
                &new_relative,
            )));
        }
        ensure_parent_dir(&new_absolute)?;
        rename_path(&old_absolute, &new_absolute)?;
        self.rename_script_sidecar_dirs(&old_relative, &new_relative)?;
        self.load_script(new_relative)
    }

    pub fn delete_script(&self, script_path: impl AsRef<Path>) -> Result<()> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        let absolute_path = self.script_absolute_path(&relative_path);
        if !absolute_path.exists() {
            return Err(StorageError::ScriptNotFound(display_path(&relative_path)));
        }
        fs::remove_file(&absolute_path).map_err(|source| StorageError::Delete {
            path: absolute_path,
            source,
        })?;
        remove_dir_if_exists(self.script_sidecar_dir("executions", &relative_path))?;
        remove_dir_if_exists(self.script_sidecar_dir("logs", &relative_path))?;
        Ok(())
    }

    pub fn dirty_state(
        &self,
        script_path: impl AsRef<Path>,
        edited_source: &str,
    ) -> Result<ScriptDirtyState> {
        let script = self.load_script(script_path)?;
        Ok(ScriptDirtyState {
            relative_path: script.relative_path,
            dirty: script.source != edited_source,
        })
    }

    pub fn save_execution(
        &self,
        script_path: impl AsRef<Path>,
        execution: &ScriptExecution,
    ) -> Result<()> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        let path = self
            .script_sidecar_dir("executions", &relative_path)
            .join(format!("{}.json", execution.execution_id));
        ensure_parent_dir(&path)?;
        let json =
            serde_json::to_string_pretty(execution).map_err(|source| StorageError::Json {
                path: path.clone(),
                source,
            })?;
        write_text(&path, &json)
    }

    pub fn load_executions(&self, script_path: impl AsRef<Path>) -> Result<Vec<ScriptExecution>> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        let dir = self.script_sidecar_dir("executions", &relative_path);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut executions: Vec<ScriptExecution> = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|source| StorageError::Read {
            path: dir.clone(),
            source,
        })? {
            let path = entry
                .map_err(|source| StorageError::Read {
                    path: dir.clone(),
                    source,
                })?
                .path();
            if path.extension() == Some(OsStr::new("json")) {
                executions.push(crate::error::read_json(path)?);
            }
        }
        executions.sort_by(|left, right| left.execution_id.cmp(&right.execution_id));
        Ok(executions)
    }

    pub fn append_log_record(
        &self,
        script_path: impl AsRef<Path>,
        record: &ScriptLogRecord,
    ) -> Result<()> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        let path = self
            .script_sidecar_dir("logs", &relative_path)
            .join(format!("{}.log", record.execution_id));
        ensure_parent_dir(&path)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|source| StorageError::Write {
                path: path.clone(),
                source,
            })?;
        writeln!(file, "{}", record.to_persisted_line())
            .map_err(|source| StorageError::Write { path, source })
    }

    pub fn load_log(
        &self,
        script_path: impl AsRef<Path>,
        execution_id: &str,
        max_records: usize,
    ) -> Result<BoundedScriptLog> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        let path = self
            .script_sidecar_dir("logs", &relative_path)
            .join(format!("{execution_id}.log"));
        let mut log = BoundedScriptLog::new(execution_id, max_records);
        if !path.exists() {
            return Ok(log);
        }
        let text = crate::error::read_to_string(path)?;
        for (index, line) in text.lines().enumerate() {
            log.push(ScriptLogRecord::from_persisted_line(
                execution_id,
                index as u64,
                line,
            ));
        }
        Ok(log)
    }

    pub fn delete_execution_artifacts(
        &self,
        script_path: impl AsRef<Path>,
        execution_id: &str,
    ) -> Result<()> {
        self.recover_live_root()?;
        let relative_path = normalize_script_path(script_path.as_ref())?;
        remove_file_if_exists(
            self.script_sidecar_dir("executions", &relative_path)
                .join(format!("{execution_id}.json")),
        )?;
        remove_file_if_exists(
            self.script_sidecar_dir("logs", &relative_path)
                .join(format!("{execution_id}.log")),
        )?;
        Ok(())
    }

    fn scripts_root(&self) -> PathBuf {
        self.scripts_root.clone()
    }

    fn script_absolute_path(&self, relative_path: &Path) -> PathBuf {
        self.scripts_root().join(relative_path)
    }

    fn script_sidecar_dir(&self, sidecar: &str, relative_path: &Path) -> PathBuf {
        self.scripts_root().join(sidecar).join(relative_path)
    }

    fn with_scripts_root(scripts_root: PathBuf) -> Self {
        Self {
            scripts_root,
            #[cfg(test)]
            failures: Arc::new(Mutex::new(FailureInjection::default())),
        }
    }

    fn validate_snapshot(&self, snapshot: &ScriptPersistenceSnapshot) -> Result<()> {
        for file in &snapshot.files {
            normalize_script_path(&file.relative_path)?;
        }
        for execution in &snapshot.executions {
            normalize_script_path(&execution.script_path)?;
        }
        Ok(())
    }

    fn write_snapshot(&self, snapshot: &ScriptPersistenceSnapshot) -> Result<()> {
        for file in &snapshot.files {
            self.create_script(&file.relative_path, &file.source)?;
        }
        for execution in &snapshot.executions {
            self.save_execution(&execution.script_path, execution)?;
        }

        let execution_paths = snapshot
            .executions
            .iter()
            .map(|execution| {
                (
                    execution.execution_id.as_str(),
                    execution.script_path.as_path(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        for record in &snapshot.logs {
            if let Some(script_path) = execution_paths.get(record.execution_id.as_str()) {
                self.append_log_record(*script_path, record)?;
            }
        }
        Ok(())
    }

    fn rename_script_sidecar_dirs(&self, old_path: &Path, new_path: &Path) -> Result<()> {
        for sidecar in ["executions", "logs"] {
            let old_dir = self.script_sidecar_dir(sidecar, old_path);
            let new_dir = self.script_sidecar_dir(sidecar, new_path);
            if !old_dir.exists() {
                continue;
            }
            if new_dir.exists() {
                return Err(StorageError::ScriptAlreadyExists(display_path(&new_dir)));
            }
            ensure_parent_dir(&new_dir)?;
            rename_path(&old_dir, &new_dir)?;
        }
        Ok(())
    }
}
