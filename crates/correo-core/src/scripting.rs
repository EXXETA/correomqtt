use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    time::{Duration, Instant},
};

use correo_scripting::{
    ScriptCancellationToken, ScriptExecutionRequest, ScriptHost, ScriptLogEntry, ScriptMqttClient,
    ScriptRuntime, ScriptingError,
};
use correo_storage::current::{
    ScriptExecution as StoredExecution, ScriptExecutionError as StoredExecutionError,
    ScriptExecutionErrorType, ScriptExecutionStatus as StoredExecutionStatus, ScriptLogRecord,
    ScriptStore,
};
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{
    MqttCommand, MqttCommandSender, ScriptExecutionError, ScriptExecutionErrorKind,
    ScriptExecutionStatus, ScriptLogLevel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptingCommand {
    Create {
        path: String,
        source: String,
    },
    Save {
        path: String,
        source: String,
    },
    Rename {
        old_path: String,
        new_path: String,
    },
    Delete {
        path: String,
    },
    ClearFinished {
        executions: Vec<(String, String)>,
    },
    Run {
        execution_id: String,
        script_name: String,
        script_path: String,
        source: String,
        connection_id: Option<String>,
        connect_command: Option<Box<MqttCommand>>,
    },
    Cancel {
        execution_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptingAction {
    Create,
    Save,
    Rename,
    Delete,
    ClearFinished,
    Run,
    Cancel,
}

impl ScriptingAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Create => "create script",
            Self::Save => "save script",
            Self::Rename => "rename script",
            Self::Delete => "delete script",
            Self::ClearFinished => "clear finished script logs",
            Self::Run => "run script",
            Self::Cancel => "cancel script",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptingEvent {
    Stored {
        action: ScriptingAction,
        path: String,
    },
    Failed {
        action: ScriptingAction,
        path: String,
        error: String,
    },
    LogAppended {
        execution_id: String,
        level: ScriptLogLevel,
        message: String,
        timestamp: String,
    },
    ExecutionUpdated {
        execution_id: String,
        status: ScriptExecutionStatus,
        duration: String,
        error: Option<ScriptExecutionError>,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScriptingDispatchError {
    #[error("scripting worker is stopped")]
    Stopped,
}

#[derive(Debug)]
pub struct ScriptingWorker {
    sender: Sender<ScriptingCommand>,
    events: Receiver<ScriptingEvent>,
}

impl ScriptingWorker {
    pub fn start(root: impl Into<PathBuf>) -> Self {
        Self::start_with_mqtt_sender(root, None)
    }

    pub fn start_with_mqtt_sender(
        root: impl Into<PathBuf>,
        mqtt_sender: Option<MqttCommandSender>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let store = ScriptStore::new(root.into());
        std::thread::spawn(move || run_worker(store, receiver, events_sender, mqtt_sender));
        Self { sender, events }
    }

    pub fn dispatch(&self, command: ScriptingCommand) -> Result<(), ScriptingDispatchError> {
        self.sender
            .send(command)
            .map_err(|_| ScriptingDispatchError::Stopped)
    }

    pub fn try_recv_event(&self) -> Option<ScriptingEvent> {
        self.events.try_recv().ok()
    }

    pub fn recv_event_timeout(&self, timeout: Duration) -> Option<ScriptingEvent> {
        self.events.recv_timeout(timeout).ok()
    }
}

fn run_worker(
    store: ScriptStore,
    receiver: Receiver<ScriptingCommand>,
    events: Sender<ScriptingEvent>,
    mqtt_sender: Option<MqttCommandSender>,
) {
    let mut cancellations = HashMap::<String, ScriptCancellationToken>::new();
    while let Ok(command) = receiver.recv() {
        match command {
            ScriptingCommand::Create { path, source } => {
                emit_storage_result(&events, ScriptingAction::Create, path.clone(), || {
                    store.create_script(&path, &source).map(|_| ())
                });
            }
            ScriptingCommand::Save { path, source } => {
                emit_storage_result(&events, ScriptingAction::Save, path.clone(), || {
                    store.update_script(&path, &source).map(|_| ())
                });
            }
            ScriptingCommand::Rename { old_path, new_path } => {
                emit_storage_result(&events, ScriptingAction::Rename, new_path.clone(), || {
                    store.rename_script(&old_path, &new_path).map(|_| ())
                });
            }
            ScriptingCommand::Delete { path } => {
                emit_storage_result(&events, ScriptingAction::Delete, path.clone(), || {
                    store.delete_script(&path)
                });
            }
            ScriptingCommand::ClearFinished { executions } => {
                emit_storage_result(
                    &events,
                    ScriptingAction::ClearFinished,
                    "finished executions".to_owned(),
                    || {
                        for (script_path, execution_id) in &executions {
                            store.delete_execution_artifacts(script_path, execution_id)?;
                        }
                        Ok(())
                    },
                );
            }
            ScriptingCommand::Run {
                execution_id,
                script_name,
                script_path,
                source,
                connection_id,
                connect_command,
            } => {
                let cancellation = ScriptCancellationToken::new();
                cancellations.insert(execution_id.clone(), cancellation.clone());
                spawn_script_run(
                    store.clone(),
                    events.clone(),
                    execution_id,
                    script_name,
                    script_path,
                    source,
                    connection_id,
                    connect_command.map(|command| *command),
                    mqtt_sender.clone(),
                    cancellation,
                );
            }
            ScriptingCommand::Cancel { execution_id } => {
                if let Some(cancellation) = cancellations.remove(&execution_id) {
                    cancellation.cancel();
                    let _ = events.send(ScriptingEvent::Stored {
                        action: ScriptingAction::Cancel,
                        path: execution_id,
                    });
                }
            }
        }
    }
}

fn spawn_script_run(
    store: ScriptStore,
    events: Sender<ScriptingEvent>,
    execution_id: String,
    script_name: String,
    script_path: String,
    source: String,
    connection_id: Option<String>,
    connect_command: Option<MqttCommand>,
    mqtt_sender: Option<MqttCommandSender>,
    cancellation: ScriptCancellationToken,
) {
    std::thread::spawn(move || {
        let started = Instant::now();
        let started_at = timestamp();
        let running = stored_execution(
            &execution_id,
            &script_name,
            &script_path,
            connection_id.clone(),
            StoredExecutionStatus::Running,
            None,
            Some(started_at.clone()),
            None,
        );
        emit_storage_result(&events, ScriptingAction::Run, script_path.clone(), || {
            store.save_execution(&script_path, &running)
        });

        let host = Arc::new(WorkerScriptHost::new(
            store.clone(),
            events.clone(),
            execution_id.clone(),
            script_path.clone(),
            crate::scripting_mqtt::client(connection_id.clone(), mqtt_sender, connect_command),
        ));
        let outcome = ScriptRuntime::new(host).execute(
            ScriptExecutionRequest::new(script_name.clone(), source),
            cancellation,
        );
        let duration_ms = started.elapsed().as_millis() as u64;
        let error = outcome.error.as_ref().map(script_error);
        let status = status_for_error(outcome.error.as_ref());
        let stored = stored_execution(
            &execution_id,
            &script_name,
            &script_path,
            connection_id,
            stored_status(status),
            outcome.error.as_ref().map(stored_error),
            Some(started_at),
            Some(duration_ms),
        );
        emit_storage_result(&events, ScriptingAction::Run, script_path.clone(), || {
            store.save_execution(&script_path, &stored)
        });
        let _ = events.send(ScriptingEvent::ExecutionUpdated {
            execution_id,
            status,
            duration: format_duration(duration_ms),
            error,
        });
    });
}

struct WorkerScriptHost {
    store: ScriptStore,
    events: Sender<ScriptingEvent>,
    execution_id: String,
    script_path: String,
    sequence: AtomicU64,
    mqtt_client: Option<Arc<dyn ScriptMqttClient>>,
}

impl WorkerScriptHost {
    fn new(
        store: ScriptStore,
        events: Sender<ScriptingEvent>,
        execution_id: String,
        script_path: String,
        mqtt_client: Option<Arc<dyn ScriptMqttClient>>,
    ) -> Self {
        Self {
            store,
            events,
            execution_id,
            script_path,
            sequence: AtomicU64::new(0),
            mqtt_client,
        }
    }
}

impl ScriptHost for WorkerScriptHost {
    fn mqtt_client(&self) -> Option<Arc<dyn ScriptMqttClient>> {
        self.mqtt_client.clone()
    }

    fn log(&self, entry: ScriptLogEntry) {
        let timestamp = timestamp();
        let level = script_log_level(entry.level);
        let record = ScriptLogRecord {
            execution_id: self.execution_id.clone(),
            sequence: self.sequence.fetch_add(1, Ordering::SeqCst),
            timestamp: Some(timestamp.clone()),
            level: stored_log_level(level),
            message: entry.message.clone(),
        };
        if let Err(error) = self.store.append_log_record(&self.script_path, &record) {
            let _ = self.events.send(ScriptingEvent::Failed {
                action: ScriptingAction::Run,
                path: self.script_path.clone(),
                error: error.to_string(),
            });
        }
        let _ = self.events.send(ScriptingEvent::LogAppended {
            execution_id: self.execution_id.clone(),
            level,
            message: entry.message,
            timestamp,
        });
    }
}

fn emit_storage_result(
    events: &Sender<ScriptingEvent>,
    action: ScriptingAction,
    path: String,
    apply: impl FnOnce() -> correo_storage::Result<()>,
) {
    let event = match apply() {
        Ok(()) => ScriptingEvent::Stored { action, path },
        Err(error) => ScriptingEvent::Failed {
            action,
            path,
            error: error.to_string(),
        },
    };
    let _ = events.send(event);
}

fn stored_execution(
    execution_id: &str,
    script_name: &str,
    script_path: &str,
    connection_id: Option<String>,
    status: StoredExecutionStatus,
    error: Option<StoredExecutionError>,
    started_at: Option<String>,
    duration_ms: Option<u64>,
) -> StoredExecution {
    StoredExecution {
        execution_id: execution_id.to_owned(),
        script_name: script_name.to_owned(),
        script_path: PathBuf::from(script_path),
        connection_id,
        status,
        error,
        started_at,
        ended_at: status.is_terminal().then(timestamp),
        duration_ms,
        cancelled: status == StoredExecutionStatus::Cancelled,
        log_path: Some(
            PathBuf::from("logs")
                .join(script_path)
                .join(format!("{execution_id}.log")),
        ),
    }
}

fn status_for_error(error: Option<&ScriptingError>) -> ScriptExecutionStatus {
    match error {
        None => ScriptExecutionStatus::Succeeded,
        Some(ScriptingError::Cancelled) => ScriptExecutionStatus::Cancelled,
        Some(_) => ScriptExecutionStatus::Failed,
    }
}

fn stored_status(status: ScriptExecutionStatus) -> StoredExecutionStatus {
    match status {
        ScriptExecutionStatus::Queued => StoredExecutionStatus::Queued,
        ScriptExecutionStatus::Running => StoredExecutionStatus::Running,
        ScriptExecutionStatus::Succeeded => StoredExecutionStatus::Succeeded,
        ScriptExecutionStatus::Failed => StoredExecutionStatus::Failed,
        ScriptExecutionStatus::Cancelled => StoredExecutionStatus::Cancelled,
    }
}

fn script_error(error: &ScriptingError) -> ScriptExecutionError {
    ScriptExecutionError {
        kind: match error {
            ScriptingError::HostApi(_) => ScriptExecutionErrorKind::HostApi,
            ScriptingError::JavaScriptGuest(_) => ScriptExecutionErrorKind::JavaScriptGuest,
            ScriptingError::Cancelled => ScriptExecutionErrorKind::Cancellation,
            ScriptingError::MqttOperation(_) => ScriptExecutionErrorKind::MqttOperation,
            ScriptingError::Runtime(_) => ScriptExecutionErrorKind::Runtime,
        },
        message: error.to_string(),
    }
}

fn stored_error(error: &ScriptingError) -> StoredExecutionError {
    let error_type = match error {
        ScriptingError::JavaScriptGuest(_) => ScriptExecutionErrorType::Guest,
        _ => ScriptExecutionErrorType::Host,
    };
    StoredExecutionError {
        error_type,
        message: error.to_string(),
    }
}

fn script_log_level(level: correo_scripting::ScriptLogLevel) -> ScriptLogLevel {
    match level {
        correo_scripting::ScriptLogLevel::Debug => ScriptLogLevel::Debug,
        correo_scripting::ScriptLogLevel::Info => ScriptLogLevel::Info,
        correo_scripting::ScriptLogLevel::Warning => ScriptLogLevel::Warning,
        correo_scripting::ScriptLogLevel::Error => ScriptLogLevel::Error,
    }
}

fn stored_log_level(level: ScriptLogLevel) -> correo_storage::current::ScriptLogLevel {
    match level {
        ScriptLogLevel::Debug => correo_storage::current::ScriptLogLevel::Debug,
        ScriptLogLevel::Info => correo_storage::current::ScriptLogLevel::Info,
        ScriptLogLevel::Warning => correo_storage::current::ScriptLogLevel::Warn,
        ScriptLogLevel::Error => correo_storage::current::ScriptLogLevel::Error,
    }
}

fn timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

fn format_duration(duration_ms: u64) -> String {
    if duration_ms < 1000 {
        return format!("{duration_ms} ms");
    }
    let seconds = duration_ms / 1000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

trait StoredStatusExt {
    fn is_terminal(&self) -> bool;
}

impl StoredStatusExt for StoredExecutionStatus {
    fn is_terminal(&self) -> bool {
        matches!(
            self,
            StoredExecutionStatus::Succeeded
                | StoredExecutionStatus::Failed
                | StoredExecutionStatus::Cancelled
        )
    }
}
