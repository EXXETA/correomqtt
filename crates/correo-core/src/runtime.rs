use std::{collections::VecDeque, sync::Arc};

use crate::{
    AppCommand, AppCommandSender, AppEvent, AppEventSender, AppModel, AppSnapshot,
    BuiltInBrokerPersistenceSnapshot, BuiltInBrokerSnapshot, BuiltInBrokerWorker, Diagnostic,
    HistoryPersistenceEvent, HistoryPersistenceKind, HistoryPersistenceWorker,
    MigrationPersistenceCommand, MigrationPersistenceWorker, MqttCommandSender, MqttService,
    NoopPluginHookExecutor, PluginHookExecutor, PluginInstaller, ScriptingWorker,
    SettingsPersistenceCommand, SettingsPersistenceEvent, SettingsPersistenceWorker, StartupState,
};

#[path = "runtime/incoming_plugins.rs"]
mod incoming_plugins;
#[path = "runtime/plugin_helpers.rs"]
mod plugin_helpers;
#[path = "runtime/plugins.rs"]
mod plugins;
#[path = "runtime/scripting.rs"]
mod scripting;
#[cfg(test)]
#[path = "runtime_test_support.rs"]
mod test_support;

const APP_COMMAND_CAPACITY: usize = 256;
const PLUGIN_SAVE_PAYLOAD_CAPACITY: usize = 1;
const APP_EVENT_CAPACITY: usize = 256;
const PUMP_BUDGET: usize = 64;

#[derive(Debug)]
pub struct AppRuntime {
    model: AppModel,
    command_sender: AppCommandSender,
    command_receiver: flume::Receiver<AppCommand>,
    event_sender: AppEventSender,
    event_receiver: flume::Receiver<AppEvent>,
    mqtt_service: Option<MqttService>,
    broker_worker: BuiltInBrokerWorker,
    history_worker: Option<HistoryPersistenceWorker>,
    migration_worker: Option<MigrationPersistenceWorker>,
    plugin_hooks: Arc<dyn PluginHookExecutor>,
    pending_plugin_save_payloads: VecDeque<crate::PluginSavePayload>,
    incoming_plugin_worker: Option<incoming_plugins::IncomingPluginWorker>,
    plugin_installer: Option<Arc<dyn PluginInstaller>>,
    settings_worker: Option<SettingsPersistenceWorker>,
    scripting_worker: Option<ScriptingWorker>,
    shutdown_requested: bool,
}

impl AppRuntime {
    pub fn new() -> Self {
        Self::with_snapshot(crate::sample_snapshot(crate::ThemeMode::System))
    }

    pub fn with_snapshot(snapshot: AppSnapshot) -> Self {
        Self::with_model(AppModel::with_snapshot(snapshot))
    }

    pub fn with_startup_state(state: StartupState) -> Self {
        Self::with_model(AppModel::with_startup_state(state))
    }

    fn with_model(model: AppModel) -> Self {
        let (command_sender, command_receiver) = flume::bounded(APP_COMMAND_CAPACITY);
        let (event_sender, event_receiver) = flume::bounded(APP_EVENT_CAPACITY);
        let app_event_sender = AppEventSender::new(event_sender);
        Self {
            model,
            command_sender: AppCommandSender::new(command_sender),
            command_receiver,
            event_sender: app_event_sender.clone(),
            event_receiver,
            mqtt_service: None,
            broker_worker: BuiltInBrokerWorker::new(app_event_sender),
            history_worker: None,
            migration_worker: None,
            plugin_hooks: Arc::new(NoopPluginHookExecutor),
            pending_plugin_save_payloads: VecDeque::new(),
            incoming_plugin_worker: None,
            plugin_installer: None,
            settings_worker: None,
            scripting_worker: None,
            shutdown_requested: false,
        }
    }

    pub fn snapshot(&self) -> &AppSnapshot {
        self.model.snapshot()
    }

    pub fn take_plugin_save_payload(&mut self) -> Option<crate::PluginSavePayload> {
        self.pending_plugin_save_payloads.pop_front()
    }

    pub fn command_sender(&self) -> AppCommandSender {
        self.command_sender.clone()
    }

    pub fn event_sender(&self) -> AppEventSender {
        self.event_sender.clone()
    }

    pub fn attach_mqtt_service(&mut self, service: MqttService) {
        self.mqtt_service = Some(service);
    }

    pub async fn shutdown_mqtt(&mut self) {
        if let Some(service) = self.mqtt_service.take() {
            service.shutdown().await;
        }
    }

    pub fn attach_history_worker(&mut self, worker: HistoryPersistenceWorker) {
        self.history_worker = Some(worker);
    }

    pub fn attach_migration_worker(&mut self, worker: MigrationPersistenceWorker) {
        self.migration_worker = Some(worker);
    }

    pub fn attach_plugin_hook_executor(&mut self, executor: impl PluginHookExecutor) {
        let executor = Arc::new(executor);
        self.incoming_plugin_worker = Some(incoming_plugins::IncomingPluginWorker::start(
            executor.clone(),
        ));
        self.plugin_hooks = executor;
    }

    pub fn attach_plugin_installer(&mut self, installer: impl PluginInstaller) {
        self.plugin_installer = Some(Arc::new(installer));
    }

    pub fn attach_settings_worker(&mut self, worker: SettingsPersistenceWorker) {
        self.settings_worker = Some(worker);
    }

    pub fn attach_scripting_worker(&mut self, worker: ScriptingWorker) {
        self.scripting_worker = Some(worker);
    }
    pub fn mqtt_command_sender(&self) -> Option<MqttCommandSender> {
        self.mqtt_service.as_ref().map(MqttService::command_sender)
    }

    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    pub fn pump(&mut self) -> PumpReport {
        let revision_before = self.model.revision();
        let mut report = PumpReport::default();
        let mut event_budget = if self.command_receiver.is_empty() {
            PUMP_BUDGET
        } else {
            PUMP_BUDGET - 1
        };

        while event_budget > 0 {
            let Some(result) = self.try_recv_incoming_hook_result() else {
                break;
            };
            if let Some(event) = result.event {
                self.dispatch_history_for_mqtt_event(&event);
                self.model.apply_event(AppEvent::Mqtt(event));
                self.append_incoming_diagnostics(result.diagnostics);
                self.refresh_message_detail(false);
                self.refresh_plugin_windows();
                self.dispatch_dirty_workbenches();
            } else {
                let message = if result.diagnostics.is_empty() {
                    "Incoming plugin processing cancelled or dropped a message.".to_owned()
                } else {
                    result
                        .diagnostics
                        .into_iter()
                        .map(|diagnostic| crate::redact_sensitive(&diagnostic.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                };
                let _ = self
                    .event_sender
                    .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(message)));
            }
            report.events_processed += 1;
            event_budget -= 1;
        }

        while event_budget > 0 {
            let Some(event) = self.try_recv_mqtt_event() else {
                break;
            };
            if self.queue_incoming_hook_job(&event) {
                report.events_processed += 1;
                event_budget -= 1;
                continue;
            }
            let Some((event, incoming_diagnostics)) = self.apply_incoming_hooks(event) else {
                report.events_processed += 1;
                event_budget -= 1;
                continue;
            };
            let refresh_detail = matches!(event, crate::MqttEvent::IncomingMessage(_));
            self.dispatch_history_for_mqtt_event(&event);
            self.model.apply_event(AppEvent::Mqtt(event));
            self.append_incoming_diagnostics(incoming_diagnostics);
            if refresh_detail {
                self.refresh_message_detail(false);
                self.refresh_plugin_windows();
            }
            self.dispatch_dirty_workbenches();
            report.events_processed += 1;
            event_budget -= 1;
        }

        self.broker_worker.poll();

        while event_budget > 0 {
            let Some(event) = self.try_recv_history_event() else {
                break;
            };
            self.apply_history_event(event);
            report.events_processed += 1;
            event_budget -= 1;
        }

        while event_budget > 0 {
            let Some(event) = self.try_recv_settings_event() else {
                break;
            };
            self.apply_settings_event(event);
            report.events_processed += 1;
            event_budget -= 1;
        }

        while event_budget > 0 {
            let Some(event) = self.try_recv_scripting_event() else {
                break;
            };
            self.apply_scripting_event(event);
            report.events_processed += 1;
            event_budget -= 1;
        }

        while event_budget > 0 {
            let Some(event) = self.try_recv_migration_event() else {
                break;
            };
            self.model.apply_event(event);
            report.events_processed += 1;
            event_budget -= 1;
        }

        while event_budget > 0 {
            let Ok(event) = self.event_receiver.try_recv() else {
                break;
            };
            self.model.apply_event(event);
            report.events_processed += 1;
            event_budget -= 1;
        }

        while report.commands_processed + report.events_processed < PUMP_BUDGET {
            let Ok(command) = self.command_receiver.try_recv() else {
                break;
            };
            let built_in_broker_before = self
                .should_persist_built_in_broker_for_command(&command)
                .then(|| self.model.snapshot().built_in_broker.clone());
            let scripting_before = command_needs_scripting_before_snapshot(&command)
                .then(|| self.model.snapshot().clone());
            let deleted_connection_id = if matches!(command, AppCommand::ConfirmDeleteConnection) {
                self.model
                    .snapshot()
                    .selected_connection
                    .map(|connection_id| self.model.storage_connection_id(connection_id))
            } else {
                None
            };
            let should_persist_settings = (matches!(command, AppCommand::SaveGlobalSettings)
                && self.model.snapshot().global_settings.dirty)
                || matches!(
                    command,
                    AppCommand::SetPluginEnabled { .. }
                        | AppCommand::ConfirmPluginDisable
                        | AppCommand::SetPluginHookEnabled { .. }
                        | AppCommand::ApplyPluginHookEdit
                );
            if matches!(command, AppCommand::Shutdown) {
                self.shutdown_requested = true;
                if let Some(worker) = &self.incoming_plugin_worker {
                    worker.cancel();
                }
            }
            self.forward_mqtt_commands(&command);
            self.forward_broker_command(&command);
            self.apply_plugin_connection_command(&command);
            self.forward_migration_command(&command);
            let plugin_file_result = self.apply_plugin_file_command(&command);
            if !plugin_file_result.proceed {
                report.commands_processed += 1;
                continue;
            }
            self.dispatch_history_for_app_command(&command);
            self.model.apply_command(command.clone());
            if let Some((plugin_id, installed_path)) = plugin_file_result.installed_path {
                self.model
                    .set_plugin_installed_path(&plugin_id, installed_path);
            }
            let allow_detail_host_actions =
                matches!(&command, AppCommand::SelectDetailTransform(Some(_)));
            if self.should_refresh_detail_for_command(&command) {
                self.refresh_message_detail(allow_detail_host_actions);
            }
            if should_persist_settings {
                self.dispatch_global_settings_save();
            }
            if matches!(command, AppCommand::SaveConnectionSettings)
                && !self.model.snapshot().connection_settings.dirty
            {
                self.dispatch_connection_settings_save();
            }
            if matches!(command, AppCommand::StartConnectionImport) {
                self.dispatch_connection_import_save();
            }
            if let Some(connection_id) = deleted_connection_id {
                self.dispatch_connection_delete(connection_id);
            }
            if matches!(command, AppCommand::MoveConnection { .. }) {
                self.dispatch_connection_order_save();
            }
            if matches!(command, AppCommand::SaveConnectionPlugins) {
                self.dispatch_connection_plugin_workflows_save();
            }
            if self.should_persist_built_in_broker_change(built_in_broker_before.as_ref()) {
                self.dispatch_built_in_broker_save();
            }
            if let Some(scripting_before) = scripting_before.as_ref() {
                self.dispatch_scripting_command(&command, scripting_before);
            }
            self.dispatch_dirty_workbenches();
            report.commands_processed += 1;
        }

        report.snapshot_changed = revision_before != self.model.revision();
        report.backlog_remaining = !self.command_receiver.is_empty()
            || !self.event_receiver.is_empty()
            || self
                .incoming_plugin_worker
                .as_ref()
                .is_some_and(incoming_plugins::IncomingPluginWorker::has_pending_results)
            || self
                .mqtt_service
                .as_ref()
                .is_some_and(MqttService::has_pending_events);
        report.shutdown_requested = self.shutdown_requested;
        report
    }
}

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

impl AppRuntime {
    fn should_persist_built_in_broker_for_command(&self, command: &AppCommand) -> bool {
        matches!(
            command,
            AppCommand::UpdateBuiltInBrokerPort(_)
                | AppCommand::SetBuiltInBrokerCredentialsEnabled(_)
                | AppCommand::UpdateBuiltInBrokerUsername(_)
                | AppCommand::UpdateBuiltInBrokerPassword(_)
        )
    }

    fn should_persist_built_in_broker_change(
        &self,
        before: Option<&BuiltInBrokerSnapshot>,
    ) -> bool {
        before.is_some_and(|before| before != &self.model.snapshot().built_in_broker)
    }

    fn dispatch_built_in_broker_save(&self) {
        let Some(worker) = &self.settings_worker else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "Settings persistence worker is not running.",
                )));
            return;
        };
        let broker = &self.model.snapshot().built_in_broker;
        if let Err(error) = worker.dispatch(SettingsPersistenceCommand::SaveBuiltInBroker {
            broker: BuiltInBrokerPersistenceSnapshot {
                port: broker.port.clone(),
                credentials_enabled: broker.credentials_enabled,
                username: broker.username.clone(),
                password: broker.password.clone(),
            },
        }) {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    error.to_string(),
                )));
        }
    }

    fn apply_settings_event(&self, event: SettingsPersistenceEvent) {
        let diagnostic = match event {
            SettingsPersistenceEvent::Saved => Diagnostic::info("Settings persisted."),
            SettingsPersistenceEvent::Failed { error } => {
                Diagnostic::error(format!("Settings persistence failed: {error}"))
            }
        };
        let _ = self
            .event_sender
            .emit(AppEvent::DiagnosticRaised(diagnostic));
    }

    fn forward_migration_command(&self, command: &AppCommand) {
        let Some(command) = self.migration_command_for_app_command(command) else {
            return;
        };
        let Some(worker) = &self.migration_worker else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "Migration worker is not running.",
                )));
            return;
        };
        if let Err(error) = worker.dispatch(command) {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    error.to_string(),
                )));
        }
    }

    fn migration_command_for_app_command(
        &self,
        command: &AppCommand,
    ) -> Option<MigrationPersistenceCommand> {
        let AppCommand::MigrationRecovery(command) = command else {
            return None;
        };
        match command {
            crate::MigrationRecoveryCommand::ChooseMigrate => self
                .model
                .snapshot()
                .migration_recovery
                .legacy_path
                .as_ref()
                .map(|legacy_path| MigrationPersistenceCommand::Prepare {
                    legacy_path: legacy_path.clone(),
                }),
            crate::MigrationRecoveryCommand::SubmitPassword { password } => {
                Some(MigrationPersistenceCommand::UnlockSecrets {
                    master_password: password.clone(),
                })
            }
            crate::MigrationRecoveryCommand::SkipSecrets => {
                Some(MigrationPersistenceCommand::SkipSecrets)
            }
            crate::MigrationRecoveryCommand::ApplyMigration => {
                Some(MigrationPersistenceCommand::Apply {
                    fallback_theme: self.model.snapshot().theme_mode.clone(),
                })
            }
            crate::MigrationRecoveryCommand::ConfirmRestoreBackup => {
                let recovery = &self.model.snapshot().migration_recovery;
                Some(MigrationPersistenceCommand::Restore {
                    backup_name: recovery.backup_name.clone()?,
                    backup_path_hint: recovery.backup_path_hint.clone()?,
                })
            }
            _ => None,
        }
    }

    fn forward_mqtt_commands(&self, command: &AppCommand) {
        // RunScript's connect command belongs to the script MQTT bridge, which
        // sends it when the script calls client.connect(). Forwarding it here
        // too would connect the same connection twice.
        if matches!(command, AppCommand::RunScript) {
            return;
        }
        let commands = match self.mqtt_commands_for_app_command_with_plugins(command) {
            Ok(commands) => commands,
            Err(error) => {
                let _ = self
                    .event_sender
                    .emit(AppEvent::DiagnosticRaised(Diagnostic::error(
                        error.to_string(),
                    )));
                return;
            }
        };
        if commands.is_empty() {
            return;
        };

        let Some(service) = &self.mqtt_service else {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                    "MQTT service is not running.",
                )));
            return;
        };

        let sender = service.command_sender();
        for command in commands {
            if let Err(error) = sender.send(command) {
                let _ = self
                    .event_sender
                    .emit(AppEvent::DiagnosticRaised(Diagnostic::error(
                        error.to_string(),
                    )));
            }
        }
    }

    fn forward_broker_command(&mut self, command: &AppCommand) {
        match command {
            AppCommand::StartBuiltInBroker => {
                if let Some(config) = self.model.broker_start_config() {
                    self.broker_worker.start(config);
                }
            }
            AppCommand::StopBuiltInBroker => self.broker_worker.stop(),
            _ => {}
        }
    }

    fn apply_plugin_file_command(&self, command: &AppCommand) -> PluginFileCommandResult {
        match command {
            AppCommand::InstallMarketplacePlugin {
                marketplace_plugin_id,
            } => {
                let Some(installer) = &self.plugin_installer else {
                    let _ =
                        self.event_sender
                            .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                                "Plugin installer is not running.",
                            )));
                    return PluginFileCommandResult::proceed();
                };
                let Some(plugin) = self
                    .model
                    .snapshot()
                    .plugins
                    .marketplace_plugins
                    .iter()
                    .find(|plugin| &plugin.id == marketplace_plugin_id)
                else {
                    return PluginFileCommandResult::proceed();
                };
                match installer.install(plugin) {
                    Ok(installed_path) => PluginFileCommandResult {
                        proceed: true,
                        installed_path: Some((marketplace_plugin_id.clone(), installed_path)),
                    },
                    Err(error) => {
                        let _ =
                            self.event_sender
                                .emit(AppEvent::DiagnosticRaised(Diagnostic::error(format!(
                                    "Plugin install failed: {error}"
                                ))));
                        PluginFileCommandResult::stop()
                    }
                }
            }
            AppCommand::UninstallPlugin { plugin_id } => {
                if self
                    .model
                    .snapshot()
                    .plugins
                    .plugins
                    .iter()
                    .find(|plugin| &plugin.id == plugin_id)
                    .is_some_and(|plugin| !plugin.can_uninstall())
                {
                    return PluginFileCommandResult::proceed();
                }
                if let Some(installer) = &self.plugin_installer {
                    if let Err(error) = installer.uninstall(plugin_id) {
                        let _ =
                            self.event_sender
                                .emit(AppEvent::DiagnosticRaised(Diagnostic::error(format!(
                                    "Plugin uninstall failed: {error}"
                                ))));
                        return PluginFileCommandResult::stop();
                    }
                }
                PluginFileCommandResult::proceed()
            }
            _ => PluginFileCommandResult::proceed(),
        }
    }
}

#[derive(Debug, Default)]
struct PluginFileCommandResult {
    proceed: bool,
    installed_path: Option<(String, String)>,
}

impl PluginFileCommandResult {
    fn proceed() -> Self {
        Self {
            proceed: true,
            installed_path: None,
        }
    }

    fn stop() -> Self {
        Self {
            proceed: false,
            installed_path: None,
        }
    }
}

fn command_needs_scripting_before_snapshot(command: &AppCommand) -> bool {
    matches!(
        command,
        AppCommand::CreateScript
            | AppCommand::SaveScript
            | AppCommand::ConfirmRenameScript
            | AppCommand::ConfirmDeleteScript
            | AppCommand::RunScript
            | AppCommand::CancelScript
            | AppCommand::ClearFinishedScriptExecutions
    )
}

fn history_kind_label(kind: HistoryPersistenceKind) -> &'static str {
    match kind {
        HistoryPersistenceKind::Publish => "Publish",
        HistoryPersistenceKind::Subscription => "Subscription",
        HistoryPersistenceKind::Workbench => "Workbench",
    }
}

impl Default for AppRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PumpReport {
    pub commands_processed: usize,
    pub events_processed: usize,
    pub backlog_remaining: bool,
    pub snapshot_changed: bool,
    pub shutdown_requested: bool,
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "runtime/plugin_tests.rs"]
mod plugin_tests;
