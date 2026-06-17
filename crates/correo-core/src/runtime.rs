use std::sync::Arc;

use crate::{
    AppCommand, AppCommandSender, AppEvent, AppEventSender, AppModel, AppSnapshot, Diagnostic,
    HistoryPersistenceEvent, HistoryPersistenceKind, HistoryPersistenceWorker,
    MigrationPersistenceCommand, MigrationPersistenceWorker, MqttCommandSender, MqttService,
    NoopPluginHookExecutor, PluginHookExecutor, PluginInstaller, ScriptingWorker,
    SettingsPersistenceCommand, SettingsPersistenceEvent, SettingsPersistenceWorker, StartupState,
};

mod plugin_helpers;
mod plugins;
mod scripting;

#[derive(Debug)]
pub struct AppRuntime {
    model: AppModel,
    command_sender: AppCommandSender,
    command_receiver: flume::Receiver<AppCommand>,
    event_sender: AppEventSender,
    event_receiver: flume::Receiver<AppEvent>,
    mqtt_service: Option<MqttService>,
    history_worker: Option<HistoryPersistenceWorker>,
    migration_worker: Option<MigrationPersistenceWorker>,
    plugin_hooks: Arc<dyn PluginHookExecutor>,
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
        let (command_sender, command_receiver) = flume::unbounded();
        let (event_sender, event_receiver) = flume::unbounded();
        Self {
            model,
            command_sender: AppCommandSender::new(command_sender),
            command_receiver,
            event_sender: AppEventSender::new(event_sender),
            event_receiver,
            mqtt_service: None,
            history_worker: None,
            migration_worker: None,
            plugin_hooks: Arc::new(NoopPluginHookExecutor),
            plugin_installer: None,
            settings_worker: None,
            scripting_worker: None,
            shutdown_requested: false,
        }
    }

    pub fn snapshot(&self) -> &AppSnapshot {
        self.model.snapshot()
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

    pub fn attach_history_worker(&mut self, worker: HistoryPersistenceWorker) {
        self.history_worker = Some(worker);
    }

    pub fn attach_migration_worker(&mut self, worker: MigrationPersistenceWorker) {
        self.migration_worker = Some(worker);
    }

    pub fn attach_plugin_hook_executor(&mut self, executor: impl PluginHookExecutor) {
        self.plugin_hooks = Arc::new(executor);
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
        let before = self.model.snapshot().clone();
        let mut report = PumpReport::default();

        while let Some(event) = self.try_recv_mqtt_event() {
            let Some((event, incoming_diagnostics)) = self.apply_incoming_hooks(event) else {
                report.events_processed += 1;
                continue;
            };
            let refresh_detail = matches!(event, crate::MqttEvent::IncomingMessage(_));
            self.dispatch_history_for_mqtt_event(&event);
            self.model.apply_event(AppEvent::Mqtt(event));
            self.append_incoming_diagnostics(incoming_diagnostics);
            if refresh_detail {
                self.refresh_message_detail();
                self.refresh_plugin_windows();
            }
            self.dispatch_dirty_workbenches();
            report.events_processed += 1;
        }

        while let Some(event) = self.try_recv_history_event() {
            self.apply_history_event(event);
            report.events_processed += 1;
        }

        while let Some(event) = self.try_recv_settings_event() {
            self.apply_settings_event(event);
            report.events_processed += 1;
        }

        while let Some(event) = self.try_recv_scripting_event() {
            self.apply_scripting_event(event);
            report.events_processed += 1;
        }

        while let Some(event) = self.try_recv_migration_event() {
            self.model.apply_event(event);
            report.events_processed += 1;
        }

        while let Ok(event) = self.event_receiver.try_recv() {
            self.model.apply_event(event);
            report.events_processed += 1;
        }

        while let Ok(command) = self.command_receiver.try_recv() {
            let command_before = self.model.snapshot().clone();
            let should_persist_settings = (matches!(command, AppCommand::SaveGlobalSettings)
                && self.model.snapshot().global_settings.dirty)
                || matches!(
                    command,
                    AppCommand::SetPluginEnabled { .. } | AppCommand::ConfirmPluginDisable
                );
            if matches!(command, AppCommand::Shutdown) {
                self.shutdown_requested = true;
            }
            self.forward_mqtt_commands(&command);
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
            if self.should_refresh_detail_for_command(&command) {
                self.refresh_message_detail();
            }
            if should_persist_settings {
                self.dispatch_global_settings_save();
            }
            if matches!(command, AppCommand::SaveConnectionPlugins) {
                self.dispatch_connection_plugin_workflows_save();
            }
            self.dispatch_scripting_command(&command, &command_before);
            self.dispatch_dirty_workbenches();
            report.commands_processed += 1;
        }

        report.snapshot_changed = before != *self.model.snapshot();
        report.shutdown_requested = self.shutdown_requested;
        report
    }

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
            settings: self.model.snapshot().global_settings.clone(),
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

    fn apply_settings_event(&self, event: SettingsPersistenceEvent) {
        let diagnostic = match event {
            SettingsPersistenceEvent::Saved => Diagnostic::info("Global settings persisted."),
            SettingsPersistenceEvent::Failed { error } => {
                Diagnostic::error(format!("Global settings persistence failed: {error}"))
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
            crate::MigrationRecoveryCommand::SubmitPassword
            | crate::MigrationRecoveryCommand::SkipSecrets => {
                Some(MigrationPersistenceCommand::LoadReview)
            }
            crate::MigrationRecoveryCommand::ApplyMigration => {
                Some(MigrationPersistenceCommand::Apply {
                    fallback_theme: self.model.snapshot().theme_mode.clone(),
                })
            }
            _ => None,
        }
    }

    fn forward_mqtt_commands(&self, command: &AppCommand) {
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
    pub snapshot_changed: bool,
    pub shutdown_requested: bool,
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    use crate::{
        AppCommand, AppEvent, AppRuntime, Diagnostic, MigrationPersistenceWorker,
        MigrationRecoveryCommand, MigrationRecoveryState, StartupState, ThemeMode,
    };

    #[test]
    fn pump_processes_commands_without_awaiting() {
        let mut runtime = AppRuntime::new();
        runtime
            .command_sender()
            .send(AppCommand::SetThemeMode(ThemeMode::Dark))
            .unwrap();

        let report = runtime.pump();

        assert_eq!(report.commands_processed, 1);
        assert!(report.snapshot_changed);
        assert_eq!(runtime.snapshot().theme_mode, ThemeMode::Dark);
    }

    #[test]
    fn pump_redacts_service_diagnostics() {
        let mut runtime = AppRuntime::new();
        runtime
            .event_sender()
            .emit(AppEvent::DiagnosticRaised(Diagnostic::warning(
                "auth failed: password:open-sesame",
            )))
            .unwrap();

        runtime.pump();

        let message = &runtime.snapshot().diagnostics[0].message;
        assert!(!message.contains("open-sesame"));
        assert!(message.contains("[REDACTED]"));
    }

    #[test]
    fn migration_worker_advances_recovery_flow_to_complete() {
        let temp = tempfile::tempdir().unwrap();
        let legacy_path = storage_fixture("legacy_profile").display().to_string();
        let mut runtime = AppRuntime::with_startup_state(StartupState::legacy_migration_detected(
            ThemeMode::Dark,
            legacy_path,
        ));
        runtime.attach_migration_worker(MigrationPersistenceWorker::start(temp.path()));

        runtime
            .command_sender()
            .send(AppCommand::MigrationRecovery(
                MigrationRecoveryCommand::ChooseMigrate,
            ))
            .unwrap();
        runtime.pump();
        assert_eq!(
            runtime.snapshot().migration_recovery.state,
            MigrationRecoveryState::CreatingBackup
        );

        pump_until(&mut runtime, |runtime| {
            runtime.snapshot().migration_recovery.state == MigrationRecoveryState::NeedsPassword
        });
        assert!(runtime.snapshot().migration_recovery.backup_name.is_some());

        runtime
            .command_sender()
            .send(AppCommand::MigrationRecovery(
                MigrationRecoveryCommand::SkipSecrets,
            ))
            .unwrap();
        runtime.pump();
        pump_until(&mut runtime, |runtime| {
            let recovery = &runtime.snapshot().migration_recovery;
            recovery.state == MigrationRecoveryState::Reviewing && recovery.counts.connections == 2
        });

        runtime
            .command_sender()
            .send(AppCommand::MigrationRecovery(
                MigrationRecoveryCommand::ApplyMigration,
            ))
            .unwrap();
        runtime.pump();
        pump_until(&mut runtime, |runtime| {
            runtime.snapshot().migration_recovery.state == MigrationRecoveryState::Complete
        });

        assert_eq!(runtime.snapshot().connection_count, 2);
        assert!(temp.path().join("config.json").exists());
    }

    fn storage_fixture(path: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../correo-storage/tests/fixtures")
            .join(path)
    }

    fn pump_until(runtime: &mut AppRuntime, mut predicate: impl FnMut(&AppRuntime) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            runtime.pump();
            if predicate(runtime) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        runtime.pump();
        assert!(predicate(runtime));
    }
}

#[cfg(test)]
mod plugin_tests;
