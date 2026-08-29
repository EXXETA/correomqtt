use crate::{
    AppCommand, AppEvent, AppSnapshot, Diagnostic, MqttCommand, ScriptExecutionRow,
    ScriptExecutionStatus, ScriptRow, ScriptingCommand, ScriptingEvent,
};

use super::AppRuntime;

impl AppRuntime {
    pub(super) fn apply_scripting_event(&mut self, event: ScriptingEvent) {
        match event {
            ScriptingEvent::Stored { action, path } => {
                self.model
                    .apply_event(AppEvent::DiagnosticRaised(Diagnostic::info(format!(
                        "{} stored for {path}.",
                        action.label()
                    ))));
            }
            ScriptingEvent::Failed {
                action,
                path,
                error,
            } => {
                self.model
                    .apply_event(AppEvent::DiagnosticRaised(Diagnostic::error(format!(
                        "{} failed for {path}: {error}",
                        action.label()
                    ))));
            }
            ScriptingEvent::LogAppended {
                execution_id,
                level,
                message,
                timestamp,
            } => {
                self.model
                    .apply_event(AppEvent::ScriptExecutionLogAppended {
                        execution_id,
                        level,
                        message,
                        timestamp,
                    });
            }
            ScriptingEvent::ExecutionUpdated {
                execution_id,
                status,
                duration,
                error,
            } => {
                self.model.apply_event(AppEvent::ScriptExecutionUpdated {
                    execution_id,
                    status,
                    duration,
                    error,
                });
                if let Some(diagnostic) = script_completion_diagnostic(status) {
                    self.model
                        .apply_event(AppEvent::DiagnosticRaised(diagnostic));
                }
            }
        }
    }

    pub(super) fn dispatch_scripting_command(&self, command: &AppCommand, before: &AppSnapshot) {
        let Some(worker) = &self.scripting_worker else {
            return;
        };
        let Some(mut command) = scripting_command(command, before, self.model.snapshot()) else {
            return;
        };
        if let ScriptingCommand::Run {
            connect_command, ..
        } = &mut command
        {
            *connect_command = self.script_connect_command().map(Box::new);
        }
        let script_started = matches!(command, ScriptingCommand::Run { .. });
        if let Err(error) = worker.dispatch(command) {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::error(
                    error.to_string(),
                )));
        } else if script_started {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(script_started_diagnostic()));
        }
    }

    fn script_connect_command(&self) -> Option<MqttCommand> {
        self.model
            .mqtt_commands_for_app_command(&AppCommand::RunScript)
            .ok()?
            .into_iter()
            .find(|command| matches!(command, MqttCommand::Connect { .. }))
    }
}

fn script_started_diagnostic() -> Diagnostic {
    Diagnostic::info("Script execution started.")
}

fn script_completion_diagnostic(status: ScriptExecutionStatus) -> Option<Diagnostic> {
    match status {
        ScriptExecutionStatus::Succeeded => Some(Diagnostic::info("Script execution succeeded.")),
        ScriptExecutionStatus::Failed => Some(Diagnostic::error("Script execution failed.")),
        _ => None,
    }
}

fn scripting_command(
    command: &AppCommand,
    before: &AppSnapshot,
    after: &AppSnapshot,
) -> Option<ScriptingCommand> {
    match command {
        AppCommand::CreateScript => selected_script(after)
            .filter(|script| !script_exists(before, &script.relative_path))
            .map(|script| ScriptingCommand::Create {
                path: script.relative_path.clone(),
                source: script.source.clone(),
            }),
        AppCommand::SaveScript => selected_script(after).map(|script| ScriptingCommand::Save {
            path: script.relative_path.clone(),
            source: script.source.clone(),
        }),
        AppCommand::ConfirmRenameScript => {
            let old_path = selected_script(before)?.relative_path.clone();
            let new_path = selected_script(after)?.relative_path.clone();
            (old_path != new_path).then_some(ScriptingCommand::Rename { old_path, new_path })
        }
        AppCommand::ConfirmDeleteScript => {
            let script = selected_script(before)?;
            if script_exists(after, &script.relative_path) {
                return None;
            }
            let path = script.relative_path.clone();
            Some(ScriptingCommand::Delete { path })
        }
        AppCommand::RunScript => {
            if !after.scripts.running
                || before.scripts.active_execution_id == after.scripts.active_execution_id
            {
                return None;
            }
            let script = selected_script(after)?;
            let execution_id = after.scripts.active_execution_id.clone()?;
            Some(ScriptingCommand::Run {
                execution_id,
                script_name: script.name.clone(),
                script_path: script.relative_path.clone(),
                source: script.source.clone(),
                connection_id: after.scripts.selected_connection_id.clone(),
                connect_command: None,
            })
        }
        AppCommand::CancelScript => {
            let execution_id = before.scripts.running_execution_id()?.to_owned();
            Some(ScriptingCommand::Cancel { execution_id })
        }
        AppCommand::ClearFinishedScriptExecutions => {
            let executions = cleared_finished_executions(before, after);
            (!executions.is_empty()).then_some(ScriptingCommand::ClearFinished { executions })
        }
        _ => None,
    }
}

fn cleared_finished_executions(before: &AppSnapshot, after: &AppSnapshot) -> Vec<(String, String)> {
    before
        .scripts
        .executions
        .iter()
        .filter(|execution| execution.status.is_terminal())
        .filter(|execution| !execution_exists(&after.scripts.executions, &execution.execution_id))
        .filter_map(|execution| {
            script_path_for(before, &execution.script_name)
                .map(|path| (path, execution.execution_id.clone()))
        })
        .collect()
}

fn execution_exists(executions: &[ScriptExecutionRow], execution_id: &str) -> bool {
    executions
        .iter()
        .any(|execution| execution.execution_id == execution_id)
}

fn script_path_for(snapshot: &AppSnapshot, script_name: &str) -> Option<String> {
    snapshot
        .scripts
        .scripts
        .iter()
        .find(|script| script.name == script_name)
        .map(|script| script.relative_path.clone())
}

fn selected_script(snapshot: &AppSnapshot) -> Option<&ScriptRow> {
    snapshot.scripts.selected_script()
}

fn script_exists(snapshot: &AppSnapshot, relative_path: &str) -> bool {
    snapshot
        .scripts
        .scripts
        .iter()
        .any(|script| script.relative_path == relative_path)
}

#[cfg(test)]
mod tests {
    use crate::{sample_snapshot, AppCommand, ScriptExecutionStatus, ScriptingCommand, ThemeMode};

    use super::{script_completion_diagnostic, script_started_diagnostic, scripting_command};

    #[test]
    fn run_script_dispatch_uses_active_execution_and_selected_connection() {
        let before = sample_snapshot(ThemeMode::Light);
        let mut after = before.clone();
        after.scripts.active_execution_id = Some("exec-1".to_owned());

        let command = scripting_command(&AppCommand::RunScript, &before, &after)
            .expect("run script should dispatch");

        assert!(matches!(
            command,
            ScriptingCommand::Run {
                execution_id,
                connection_id: Some(_),
                ..
            } if execution_id == "exec-1"
        ));
    }

    #[test]
    fn clear_finished_dispatch_deletes_removed_execution_artifacts() {
        let before = sample_snapshot(ThemeMode::Light);
        let mut after = before.clone();
        after
            .scripts
            .executions
            .retain(|execution| !execution.status.is_terminal());

        let command =
            scripting_command(&AppCommand::ClearFinishedScriptExecutions, &before, &after)
                .expect("clear finished should dispatch");

        assert!(matches!(
            command,
            ScriptingCommand::ClearFinished { executions }
                if executions == vec![
                    ("scripts/payload_replay.js".to_owned(), "exec-0999".to_owned()),
                    ("scripts/payload_replay.js".to_owned(), "exec-0998".to_owned()),
                ]
        ));
    }

    #[test]
    fn script_lifecycle_diagnostics_use_toast_messages() {
        assert_eq!(
            script_started_diagnostic().message,
            "Script execution started."
        );
        assert_eq!(
            script_completion_diagnostic(ScriptExecutionStatus::Succeeded)
                .expect("success diagnostic")
                .message,
            "Script execution succeeded."
        );
        assert_eq!(
            script_completion_diagnostic(ScriptExecutionStatus::Failed)
                .expect("failed diagnostic")
                .message,
            "Script execution failed."
        );
        assert!(script_completion_diagnostic(ScriptExecutionStatus::Cancelled).is_none());
    }
}
