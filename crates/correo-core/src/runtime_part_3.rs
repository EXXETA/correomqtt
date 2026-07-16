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
