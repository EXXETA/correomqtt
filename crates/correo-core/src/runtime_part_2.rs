impl AppRuntime {
    fn try_recv_mqtt_event(&self) -> Option<crate::MqttEvent> {
        self.mqtt_service
            .as_ref()
            .and_then(|service| service.try_recv_event().ok())
    }

    fn try_recv_history_event(&self) -> Option<HistoryPersistenceEvent> {
        self.history_worker
            .as_ref()
            .and_then(HistoryPersistenceWorker::try_recv_event)
    }

    fn try_recv_settings_event(&self) -> Option<SettingsPersistenceEvent> {
        self.settings_worker
            .as_ref()
            .and_then(SettingsPersistenceWorker::try_recv_event)
    }

    fn try_recv_scripting_event(&self) -> Option<crate::ScriptingEvent> {
        self.scripting_worker
            .as_ref()
            .and_then(ScriptingWorker::try_recv_event)
    }

    fn try_recv_migration_event(&self) -> Option<AppEvent> {
        self.migration_worker
            .as_ref()
            .and_then(MigrationPersistenceWorker::try_recv_event)
    }

    fn dispatch_history_for_mqtt_event(&self, event: &crate::MqttEvent) {
        let commands = self.model.history_commands_for_mqtt_event(event);
        if commands.is_empty() {
            return;
        }
        let Some(worker) = &self.history_worker else {
            return;
        };
        for command in commands {
            if let Err(error) = worker.dispatch(command) {
                let _ = self
                    .event_sender
                    .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                        error.to_string(),
                    )));
            }
        }
    }

    fn dispatch_history_for_app_command(&self, command: &AppCommand) {
        let commands = self.model.history_commands_for_app_command(command);
        if commands.is_empty() {
            return;
        }
        let Some(worker) = &self.history_worker else {
            return;
        };
        for command in commands {
            if let Err(error) = worker.dispatch(command) {
                let _ = self
                    .event_sender
                    .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                        error.to_string(),
                    )));
            }
        }
    }

    fn dispatch_dirty_workbenches(&mut self) {
        let commands = self.model.drain_workbench_persistence_commands();
        let Some(worker) = &self.history_worker else {
            if !commands.is_empty() {
                let _ = self
                    .event_sender
                    .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                        "Workbench persistence worker is not running.",
                    )));
            }
            return;
        };
        for command in commands {
            if let Err(error) = worker.dispatch(command) {
                let _ = self
                    .event_sender
                    .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                        error.to_string(),
                    )));
            }
        }
    }

    fn apply_history_event(&self, event: HistoryPersistenceEvent) {
        if let HistoryPersistenceEvent::Failed {
            connection_id,
            kind,
            error,
        } = event
        {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::error(format!(
                    "{} history persistence failed for {connection_id}: {error}",
                    history_kind_label(kind)
                ))));
        }
    }

    fn dispatch_global_settings_save(&self) {
        let Some(worker) = &self.settings_worker else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "Settings persistence worker is not running.",
                )));
            return;
        };
        if let Err(error) = worker.dispatch(SettingsPersistenceCommand::Save {
            theme_mode: self.model.snapshot().theme_mode.clone(),
            settings: Box::new(self.model.snapshot().global_settings.clone()),
        }) {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    error.to_string(),
                )));
        }
    }

    fn dispatch_connection_plugin_workflows_save(&self) {
        let Some(connection_id) = self.model.snapshot().selected_connection else {
            return;
        };
        let Some(worker) = &self.settings_worker else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "Settings persistence worker is not running.",
                )));
            return;
        };
        let storage_connection_id = self.model.storage_connection_id(connection_id);
        if let Err(error) =
            worker.dispatch(SettingsPersistenceCommand::SaveConnectionPluginWorkflows {
                connection_id: storage_connection_id,
                workflows: self
                    .model
                    .snapshot()
                    .connection_settings
                    .plugin_workflows
                    .clone(),
            })
        {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    error.to_string(),
                )));
        }
    }

    fn dispatch_connection_settings_save(&self) {
        let Some(connection_id) = self.model.snapshot().selected_connection else {
            return;
        };
        let Some(worker) = &self.settings_worker else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "Settings persistence worker is not running.",
                )));
            return;
        };
        let storage_connection_id = self.model.storage_connection_id(connection_id);
        if let Err(error) = worker.dispatch(SettingsPersistenceCommand::SaveConnectionSettings {
            connection_id: storage_connection_id,
            settings: Box::new(self.model.snapshot().connection_settings.clone()),
        }) {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    error.to_string(),
                )));
        }
    }

    fn dispatch_connection_delete(&self, connection_id: String) {
        let Some(worker) = &self.settings_worker else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "Settings persistence worker is not running.",
                )));
            return;
        };
        if let Err(error) =
            worker.dispatch(SettingsPersistenceCommand::DeleteConnection { connection_id })
        {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    error.to_string(),
                )));
        }
    }

    fn dispatch_connection_order_save(&self) {
        let Some(worker) = &self.settings_worker else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "Settings persistence worker is not running.",
                )));
            return;
        };
        let connection_ids = self
            .model
            .snapshot()
            .connections
            .iter()
            .map(|connection| self.model.storage_connection_id(connection.id))
            .collect();
        if let Err(error) =
            worker.dispatch(SettingsPersistenceCommand::SaveConnectionOrder { connection_ids })
        {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    error.to_string(),
                )));
        }
    }

    fn dispatch_connection_import_save(&mut self) {
        if self.settings_worker.is_none() {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "Settings persistence worker is not running.",
                )));
            return;
        }
        let Some((connections, secrets)) = self.model.drain_connection_import_persistence() else {
            return;
        };
        let worker = self.settings_worker.as_ref().expect("checked above");
        if let Err(error) = worker.dispatch(SettingsPersistenceCommand::SaveImportedConnections {
            connections,
            secrets,
        }) {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    error.to_string(),
                )));
        }
    }

}

