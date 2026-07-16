impl AppModel {
    pub(super) fn search_scripts(&mut self, filter: String) {
        self.snapshot.scripts.script_filter = filter;
    }

    pub(super) fn select_script(&mut self, name: String) {
        if self.script_index(&name).is_some() {
            self.snapshot.scripts.selected_script = name;
            self.snapshot.scripts.feedback = None;
        }
    }

    pub(super) fn select_script_connection(&mut self, connection_id: &str) {
        let Some(connection) = self
            .snapshot
            .connections
            .iter()
            .find(|connection| connection.id.to_string() == connection_id)
        else {
            self.set_script_feedback(ScriptFeedback::warning("Select an available connection."));
            return;
        };
        self.snapshot.scripts.selected_connection_id = Some(connection.id.to_string());
        self.snapshot.scripts.selected_connection = connection.name.clone();
        self.snapshot.scripts.feedback = None;
    }

    pub(super) fn request_create_script(&mut self) {
        self.snapshot.scripts.new_script_name = self.next_default_script_name();
        self.snapshot.scripts.create_error = None;
        self.snapshot.scripts.create_dialog_open = true;
    }

    pub(super) fn update_new_script_name(&mut self, name: String) {
        self.snapshot.scripts.new_script_name = name;
        self.snapshot.scripts.create_error = None;
    }

    pub(super) fn cancel_create_script(&mut self) {
        self.snapshot.scripts.create_dialog_open = false;
        self.snapshot.scripts.new_script_name.clear();
        self.snapshot.scripts.create_error = None;
    }

    pub(super) fn create_script(&mut self) {
        let draft_name = self.snapshot.scripts.new_script_name.trim();
        let name = match normalize_script_path(draft_name) {
            Ok(name) => name,
            Err(message) => {
                self.snapshot.scripts.create_error = Some(message.clone());
                self.snapshot.scripts.create_dialog_open = true;
                self.set_script_feedback(ScriptFeedback::error(message));
                return;
            }
        };
        if self.script_index(&name).is_some() {
            let message = format!("Script already exists: {name}");
            self.snapshot.scripts.create_error = Some(message.clone());
            self.snapshot.scripts.create_dialog_open = true;
            self.set_script_feedback(ScriptFeedback::error(message));
            return;
        }

        let source = default_script_source(&name);
        self.snapshot.scripts.scripts.push(ScriptRow {
            name: name.clone(),
            relative_path: name.clone(),
            status: ScriptFileStatus::Ready,
            execution_count: 0,
            saved_source: source.clone(),
            persisted: true,
            source,
        });
        self.snapshot.scripts.selected_script = name.clone();
        self.snapshot.scripts.create_dialog_open = false;
        self.snapshot.scripts.create_error = None;
        self.snapshot.scripts.new_script_name.clear();
        self.set_script_feedback(ScriptFeedback::info(format!("Created {name}.")));
    }

    fn next_default_script_name(&self) -> String {
        for index in 1.. {
            let candidate = if index == 1 {
                "new_script.js".to_owned()
            } else {
                format!("new_script_{index}.js")
            };
            if self.script_index(&candidate).is_none() {
                return candidate;
            }
        }
        unreachable!("unbounded default script name search should always find a candidate")
    }

    pub(super) fn update_script_source(&mut self, source: String) {
        let Some(index) = self.selected_script_index() else {
            return;
        };
        let script = &mut self.snapshot.scripts.scripts[index];
        script.source = source;
        if script.status != ScriptFileStatus::Running {
            script.status = if script.is_dirty() || !script.persisted {
                ScriptFileStatus::Dirty
            } else {
                ScriptFileStatus::Ready
            };
        }
    }

    pub(super) fn save_script(&mut self) {
        let Some(index) = self.selected_script_index() else {
            self.set_script_feedback(ScriptFeedback::warning("Select a script before saving."));
            return;
        };
        let script = &mut self.snapshot.scripts.scripts[index];
        script.saved_source = script.source.clone();
        script.persisted = true;
        if script.status != ScriptFileStatus::Running {
            script.status = ScriptFileStatus::Ready;
        }
        let name = script.name.clone();
        self.set_script_feedback(ScriptFeedback::info(format!("Saved {name}.")));
    }

    pub(super) fn discard_script_changes(&mut self) {
        let Some(index) = self.selected_script_index() else {
            self.set_script_feedback(ScriptFeedback::warning(
                "Select a script before discarding changes.",
            ));
            return;
        };
        let script = &mut self.snapshot.scripts.scripts[index];
        script.source = script.saved_source.clone();
        if script.status != ScriptFileStatus::Running {
            script.status = if script.persisted {
                ScriptFileStatus::Ready
            } else {
                ScriptFileStatus::Dirty
            };
        }
        let name = script.name.clone();
        self.set_script_feedback(ScriptFeedback::info(format!(
            "Discarded changes to {}.",
            name
        )));
    }

    pub(super) fn request_rename_script(&mut self) {
        if self.snapshot.scripts.selected_script().is_none() {
            self.set_script_feedback(ScriptFeedback::warning("Select a script before renaming."));
            return;
        }
        self.snapshot.scripts.rename_script_name = self.snapshot.scripts.selected_script.clone();
        self.snapshot.scripts.rename_error = None;
        self.snapshot.scripts.rename_dialog_open = true;
    }

    pub(super) fn update_rename_script_name(&mut self, name: String) {
        self.snapshot.scripts.rename_script_name = name;
        self.snapshot.scripts.rename_error = None;
    }

    pub(super) fn cancel_rename_script(&mut self) {
        self.snapshot.scripts.rename_dialog_open = false;
        self.snapshot.scripts.rename_script_name.clear();
        self.snapshot.scripts.rename_error = None;
    }

    pub(super) fn confirm_rename_script(&mut self) {
        let Some(index) = self.selected_script_index() else {
            self.snapshot.scripts.rename_dialog_open = false;
            return;
        };
        let name = match normalize_script_path(&self.snapshot.scripts.rename_script_name) {
            Ok(name) => name,
            Err(message) => {
                self.snapshot.scripts.rename_error = Some(message.clone());
                self.set_script_feedback(ScriptFeedback::error(message));
                return;
            }
        };
        if self
            .script_index(&name)
            .is_some_and(|existing| existing != index)
        {
            let message = format!("Script already exists: {name}");
            self.snapshot.scripts.rename_error = Some(message.clone());
            self.set_script_feedback(ScriptFeedback::error(message));
            return;
        }

        let old_name = self.snapshot.scripts.scripts[index].name.clone();
        self.snapshot.scripts.scripts[index].name = name.clone();
        self.snapshot.scripts.scripts[index].relative_path = name.clone();
        self.snapshot.scripts.selected_script = name.clone();
        self.snapshot.scripts.rename_dialog_open = false;
        self.snapshot.scripts.rename_error = None;
        self.set_script_feedback(ScriptFeedback::info(format!(
            "Renamed {old_name} to {name}."
        )));
    }

    pub(super) fn request_delete_script(&mut self) {
        if self.snapshot.scripts.selected_script().is_none() {
            self.set_script_feedback(ScriptFeedback::warning("Select a script before deleting."));
            return;
        }
        self.snapshot.scripts.delete_confirmation_open = true;
    }

    pub(super) fn cancel_delete_script(&mut self) {
        self.snapshot.scripts.delete_confirmation_open = false;
    }

    pub(super) fn confirm_delete_script(&mut self) {
        if self.snapshot.scripts.running {
            self.set_script_feedback(ScriptFeedback::warning(
                "Cancel the running script before deleting it.",
            ));
            return;
        }
        let Some(index) = self.selected_script_index() else {
            self.snapshot.scripts.delete_confirmation_open = false;
            return;
        };
        let deleted = self.snapshot.scripts.scripts.remove(index).name;
        self.snapshot.scripts.selected_script = self
            .snapshot
            .scripts
            .scripts
            .first()
            .map(|script| script.name.clone())
            .unwrap_or_default();
        if self.snapshot.scripts.scripts.is_empty() {
            self.snapshot.scripts.selected_execution_id = None;
        }
        self.snapshot.scripts.delete_confirmation_open = false;
        self.set_script_feedback(ScriptFeedback::warning(format!(
            "Deleted {deleted}; script sidecars will be removed by storage."
        )));
    }

    pub(super) fn select_script_detail_tab(&mut self, tab: ScriptDetailTab) {
        self.snapshot.scripts.active_tab = tab;
    }

    pub(super) fn select_script_execution(&mut self, execution_id: String) {
        if self
            .snapshot
            .scripts
            .executions
            .iter()
            .any(|execution| execution.execution_id == execution_id)
        {
            self.snapshot.scripts.selected_execution_id = Some(execution_id);
        }
    }

    pub(super) fn run_script(&mut self) {
        if self.snapshot.scripts.running {
            self.set_script_feedback(ScriptFeedback::warning(
                "A script execution is already running.",
            ));
            return;
        }
        let Some(index) = self.selected_script_index() else {
            self.set_script_feedback(ScriptFeedback::warning("Select a script before running."));
            return;
        };

        let script_name = self.snapshot.scripts.scripts[index].name.clone();
        let execution_id = format!("script-exec-{}", self.snapshot.scripts.executions.len() + 1);
        let timestamp = current_timestamp();
        self.snapshot.scripts.running = true;
        self.snapshot.scripts.active_execution_id = Some(execution_id.clone());
        self.snapshot.scripts.selected_execution_id = Some(execution_id.clone());
        self.snapshot.scripts.active_tab = ScriptDetailTab::Executions;
        self.snapshot.scripts.scripts[index].status = ScriptFileStatus::Running;
        self.snapshot.scripts.scripts[index].execution_count += 1;
        self.snapshot.scripts.executions.insert(
            0,
            ScriptExecutionRow {
                execution_id: execution_id.clone(),
                script_name: script_name.clone(),
                status: ScriptExecutionStatus::Running,
                duration: "running".to_owned(),
                timestamp: timestamp.clone(),
                error: None,
            },
        );
        self.append_script_log(
            execution_id,
            ScriptLogLevel::Info,
            format!(
                "{script_name} execution queued for {}",
                self.snapshot.scripts.selected_connection
            ),
            timestamp,
        );
    }

    pub(super) fn cancel_script(&mut self) {
        let Some(execution_id) = self
            .snapshot
            .scripts
            .running_execution_id()
            .map(str::to_owned)
        else {
            self.set_script_feedback(ScriptFeedback::warning("No running script to cancel."));
            return;
        };
        self.snapshot.scripts.active_execution_id = Some(execution_id.clone());
        self.snapshot.scripts.running = true;
        self.append_script_log(
            execution_id.clone(),
            ScriptLogLevel::Warning,
            "Cancellation requested by user.".to_owned(),
            current_timestamp(),
        );
        self.update_script_execution(
            execution_id,
            ScriptExecutionStatus::Cancelled,
            "cancelled".to_owned(),
            Some(ScriptExecutionError {
                kind: ScriptExecutionErrorKind::Cancellation,
                message: "Script execution was cancelled.".to_owned(),
            }),
        );
    }

    pub(super) fn remove_script_execution(&mut self, execution_id: &str) {
        let Some(index) = self
            .snapshot
            .scripts
            .executions
            .iter()
            .position(|execution| execution.execution_id == execution_id)
        else {
            return;
        };
        if self.snapshot.scripts.executions[index].status == ScriptExecutionStatus::Running {
            self.cancel_script();
        }
        self.snapshot.scripts.executions.remove(index);
        self.snapshot
            .scripts
            .log_lines
            .retain(|line| line.execution_id != execution_id);
        if self.snapshot.scripts.selected_execution_id.as_deref() == Some(execution_id) {
            self.snapshot.scripts.selected_execution_id = self
                .snapshot
                .scripts
                .executions
                .first()
                .map(|execution| execution.execution_id.clone());
        }
    }

    pub(super) fn append_script_log(
        &mut self,
        execution_id: String,
        level: ScriptLogLevel,
        message: String,
        timestamp: String,
    ) {
        self.snapshot.scripts.log_lines.push(ScriptLogLine {
            execution_id: execution_id.clone(),
            timestamp,
            level,
            message: redact_script_output(&format!("[{execution_id}] {message}")),
        });
        if self.snapshot.scripts.log_lines.len() > 200 {
            let overflow = self.snapshot.scripts.log_lines.len() - 200;
            self.snapshot.scripts.log_lines.drain(0..overflow);
        }
    }

    pub(super) fn update_script_execution(
        &mut self,
        execution_id: String,
        status: ScriptExecutionStatus,
        duration: String,
        error: Option<ScriptExecutionError>,
    ) {
        let redacted_error = error.map(redact_script_error);
        if let Some(execution) = self
            .snapshot
            .scripts
            .executions
            .iter_mut()
            .find(|execution| execution.execution_id == execution_id)
        {
            execution.status = status;
            execution.duration = duration;
            execution.error = redacted_error.clone();
        }

        if status.is_terminal()
            && self.snapshot.scripts.active_execution_id.as_deref() == Some(execution_id.as_str())
        {
            self.snapshot.scripts.running = false;
            self.snapshot.scripts.active_execution_id = None;
            if let Some(index) = self.selected_script_index() {
                self.snapshot.scripts.scripts[index].status = match status {
                    ScriptExecutionStatus::Failed => ScriptFileStatus::Error,
                    _ if self.snapshot.scripts.scripts[index].is_dirty() => ScriptFileStatus::Dirty,
                    _ => ScriptFileStatus::Ready,
                };
            }
        }

        if let Some(error) = redacted_error.clone() {
            self.append_script_log(
                execution_id,
                ScriptLogLevel::Error,
                format!("{}: {}", error.kind.label(), error.message),
                current_timestamp(),
            );
            self.snapshot.scripts.last_error = Some(error);
        } else if status.is_terminal() {
            self.snapshot.scripts.last_error = None;
        }
    }

    fn selected_script_index(&self) -> Option<usize> {
        self.script_index(&self.snapshot.scripts.selected_script)
    }

    fn script_index(&self, name: &str) -> Option<usize> {
        self.snapshot
            .scripts
            .scripts
            .iter()
            .position(|script| script.name == name)
    }

    pub(super) fn set_script_feedback(&mut self, feedback: ScriptFeedback) {
        let diagnostic = match feedback.severity {
            ScriptFeedbackSeverity::Info => Diagnostic::info(feedback.message.clone()),
            ScriptFeedbackSeverity::Warning => Diagnostic::warning(feedback.message.clone()),
            ScriptFeedbackSeverity::Error => Diagnostic::error(feedback.message.clone()),
        };
        self.snapshot.scripts.feedback = Some(feedback);
        self.push_diagnostic(diagnostic);
    }
}

