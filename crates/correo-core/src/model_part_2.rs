impl AppModel {
    pub fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::ConnectionListLoaded { connections } => {
                self.snapshot.connection_count = connections.len();
                self.snapshot.connections = connections;
                self.sync_built_in_broker_connection();
                if self
                    .snapshot
                    .selected_connection
                    .is_none_or(|id| self.connection_index(id).is_none())
                {
                    self.snapshot.selected_connection = self
                        .snapshot
                        .connections
                        .first()
                        .map(|connection| connection.id);
                }
                if self.snapshot.connection_surface == crate::ConnectionSurface::Launcher {
                    self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
                }
            }
            AppEvent::ConnectionOpened { connection_id } => {
                self.snapshot.active_connection = Some(connection_id);
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Connected,
                    Some(ConnectDisabledReason::AlreadyConnected),
                    "connected".to_owned(),
                );
            }
            AppEvent::ConnectionClosed { connection_id } => {
                if self.snapshot.active_connection == Some(connection_id) {
                    self.snapshot.active_connection = None;
                }
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Disconnected,
                    None,
                    "disconnected".to_owned(),
                );
            }
            AppEvent::ConnectionStateChanged {
                connection_id,
                state,
                disabled_reason,
                last_activity,
            } => {
                self.update_connection_state(connection_id, state, disabled_reason, last_activity);
            }
            AppEvent::ConnectionSettingsLoaded {
                connection_id,
                settings,
            } => {
                self.snapshot.selected_connection = Some(connection_id);
                self.connection_settings
                    .insert(connection_id, (*settings).clone());
                self.snapshot.connection_settings = *settings;
            }
            AppEvent::GlobalSettingsLoaded { settings } => self.load_global_settings(*settings),
            AppEvent::ThemeModeChanged { mode } => self.snapshot.theme_mode = mode,
            AppEvent::MigrationApplied {
                state,
                completion,
                diagnostics,
            } => self.apply_migrated_startup_state(*state, completion, diagnostics),
            AppEvent::DiagnosticRaised(diagnostic) => self.push_diagnostic(diagnostic),
            AppEvent::UpdateCheckCompleted {
                summary,
                update_available,
            } => {
                self.snapshot.global_settings.last_update_check = summary.clone();
                self.saved_global_settings.last_update_check = summary.clone();
                if update_available {
                    self.push_diagnostic(crate::Diagnostic::info(summary));
                }
            }
            AppEvent::ScriptExecutionLogAppended {
                execution_id,
                level,
                message,
                timestamp,
            } => self.append_script_log(execution_id, level, message, timestamp),
            AppEvent::ScriptExecutionUpdated {
                execution_id,
                status,
                duration,
                error,
            } => self.update_script_execution(execution_id, status, duration, error),
            AppEvent::Mqtt(event) => self.apply_mqtt_event(event),
            AppEvent::BuiltInBroker(event) => self.apply_broker_event(event),
            AppEvent::MigrationRecovery(event) => self.apply_migration_recovery_event(event),
            AppEvent::PluginWorkflow(event) => self.apply_plugin_workflow_event(event),
        }
        self.bump_revision();
    }

    fn publish_from_snapshot(&mut self) {
        if self.snapshot.active_connection.is_none() {
            self.snapshot.workbench.publish.feedback = Some(crate::WorkflowFeedback::warning(
                "Publish requires an active MQTT connection.",
            ));
            self.push_diagnostic(Diagnostic::warning(
                "Publish requires an active MQTT connection.",
            ));
            return;
        }
        let topic = self.snapshot.workbench.publish.topic.trim().to_owned();
        if topic.is_empty() {
            self.snapshot.workbench.publish.feedback = Some(crate::WorkflowFeedback::warning(
                "Publish topic is required.",
            ));
            self.push_diagnostic(Diagnostic::warning("Publish topic is required."));
            return;
        }
        if !self.snapshot.workbench.publish.valid {
            self.snapshot.workbench.publish.feedback = Some(crate::WorkflowFeedback::warning(
                "Publish topic is invalid.",
            ));
            return;
        }
        self.snapshot.workbench.publish.feedback = Some(crate::WorkflowFeedback::info(format!(
            "Publish queued for {topic}."
        )));
        self.push_diagnostic(Diagnostic::info(format!(
            "Publish command queued for {topic}."
        )));
    }

    fn subscribe_from_snapshot(&mut self) {
        if self.snapshot.active_connection.is_none() {
            self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::warning(
                "Subscribe requires an active MQTT connection.",
            ));
            self.push_diagnostic(Diagnostic::warning(
                "Subscribe requires an active MQTT connection.",
            ));
            return;
        }
        let topic = self.snapshot.workbench.subscribe.topic.trim().to_owned();
        if topic.is_empty() {
            self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::warning(
                "Subscribe topic is required.",
            ));
            self.push_diagnostic(Diagnostic::warning("Subscribe topic is required."));
            return;
        }
        if !self.snapshot.workbench.subscribe.valid {
            self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::warning(
                "Subscribe topic filter is invalid.",
            ));
            return;
        }
        self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::info(format!(
            "Subscribe queued for {topic}."
        )));
        self.push_diagnostic(Diagnostic::info(format!(
            "Subscribe command queued for {topic}."
        )));
    }

    fn unsubscribe(&mut self, topic: &str) {
        self.snapshot
            .workbench
            .subscribe
            .subscriptions
            .retain(|subscription| subscription.topic_filter != topic);
        self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::info(format!(
            "Unsubscribe queued for {topic}."
        )));
        self.push_diagnostic(Diagnostic::info(format!(
            "Unsubscribe command queued for {topic}."
        )));
    }

    fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.snapshot.diagnostics.insert(0, diagnostic.redacted());
        self.snapshot.diagnostics.truncate(12);
    }
}

impl Default for AppModel {
    fn default() -> Self {
        Self::new()
    }
}

fn command_mutates_active_workbench(command: &AppCommand) -> bool {
    matches!(
        command,
        AppCommand::ImportMessages
            | AppCommand::ImportMessagesFromPath(_)
            | AppCommand::ExportMessages
            | AppCommand::ExportPublishHistoryMessage(_)
            | AppCommand::ExportIncomingMessage(_)
            | AppCommand::ExportPublishHistoryMessageToPath { .. }
            | AppCommand::ExportIncomingMessageToPath { .. }
            | AppCommand::CopyPublishHistoryMessageToPublishForm(_)
            | AppCommand::CopyIncomingMessageToPublishForm(_)
            | AppCommand::RemovePublishHistoryMessage(_)
            | AppCommand::RemoveIncomingMessage(_)
            | AppCommand::ClearPublishHistory
            | AppCommand::ClearIncomingMessages
            | AppCommand::SelectWorkbenchTab(_)
            | AppCommand::UpdatePublishTopic(_)
            | AppCommand::UpdatePublishPayload(_)
            | AppCommand::UpdatePublishQos(_)
            | AppCommand::SetPublishRetained(_)
            | AppCommand::SearchPublishHistory(_)
            | AppCommand::SelectPublishHistoryMessage(_)
            | AppCommand::Publish
            | AppCommand::UpdateSubscribeTopic(_)
            | AppCommand::UpdateSubscribeQos(_)
            | AppCommand::Subscribe
            | AppCommand::Unsubscribe(_)
            | AppCommand::UnsubscribeAll
            | AppCommand::CancelUnsubscribeAll
            | AppCommand::ConfirmUnsubscribeAll
            | AppCommand::SetSubscriptionMessagesVisible { .. }
            | AppCommand::SetAllSubscriptionMessagesVisible(_)
            | AppCommand::SelectSubscription { .. }
            | AppCommand::SearchMessages(_)
            | AppCommand::SelectMessage(_)
            | AppCommand::SelectInspectorTab(_)
            | AppCommand::SelectDetailTransform(_)
            | AppCommand::SelectDetailFormatter(_)
            | AppCommand::RefreshMessageDetail
    )
}

#[cfg(test)]
#[path = "model/connection_list_tests.rs"]
mod connection_list_tests;
#[cfg(test)]
#[path = "model/connection_settings_tests.rs"]
mod connection_settings_tests;
#[cfg(test)]
#[path = "model/migration_secret_tests.rs"]
mod migration_secret_tests;
#[cfg(test)]
#[path = "model/plugin_tests.rs"]
mod plugin_tests;
#[cfg(test)]
#[path = "model/tests.rs"]
mod tests;
