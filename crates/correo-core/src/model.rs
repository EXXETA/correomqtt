use std::collections::{HashMap, HashSet};

use correo_mqtt::ConnectionId;
use correo_storage::current::{
    ConnectionConfig as StorageConnectionConfig, ImportedSecret as StorageImportedSecret,
};

use crate::{
    AppCommand, AppEvent, AppSnapshot, ConnectDisabledReason, ConnectionSettingsSnapshot,
    ConnectionState, Diagnostic, MqttCommand, MqttCommandBuildError, StartupState,
    WorkbenchSnapshot,
};

mod broker;
mod connections;
mod history;
mod migration_recovery;
mod mqtt;
mod plugin_workflows;
mod plugins;
mod scripting;
mod scripting_cleanup;
mod scripting_commands;
#[cfg(test)]
mod scripting_tests;
mod settings;
mod subscriptions;
mod transfer;
mod transfer_connection_export;
mod transfer_connections;

#[derive(Debug, Clone)]
pub struct AppModel {
    snapshot: AppSnapshot,
    connection_settings: HashMap<ConnectionId, ConnectionSettingsSnapshot>,
    storage_connection_ids: HashMap<ConnectionId, String>,
    workbenches: HashMap<ConnectionId, WorkbenchSnapshot>,
    dirty_workbenches: HashSet<ConnectionId>,
    saved_global_settings: crate::GlobalSettingsSnapshot,
    saved_theme_mode: crate::ThemeMode,
    pending_connection_imports: HashMap<String, StorageConnectionConfig>,
    pending_connection_import_secrets: Vec<StorageImportedSecret>,
    pending_connection_import_persistence:
        Option<(Vec<StorageConnectionConfig>, Vec<StorageImportedSecret>)>,
}

impl AppModel {
    pub fn new() -> Self {
        Self::with_snapshot(crate::sample_snapshot(crate::ThemeMode::System))
    }

    pub fn empty() -> Self {
        Self::with_snapshot(AppSnapshot::empty())
    }

    pub fn with_snapshot(snapshot: AppSnapshot) -> Self {
        Self::from_parts(snapshot, HashMap::new(), HashMap::new(), HashMap::new())
    }

    pub fn with_startup_state(state: StartupState) -> Self {
        Self::from_parts(
            state.snapshot,
            state.connection_settings,
            state.storage_connection_ids,
            state.workbenches,
        )
    }

    fn from_parts(
        snapshot: AppSnapshot,
        connection_settings: HashMap<ConnectionId, ConnectionSettingsSnapshot>,
        storage_connection_ids: HashMap<ConnectionId, String>,
        mut workbenches: HashMap<ConnectionId, WorkbenchSnapshot>,
    ) -> Self {
        let saved_global_settings = snapshot.global_settings.clone();
        let saved_theme_mode = snapshot.theme_mode.clone();
        if let Some(selected) = snapshot.selected_connection {
            workbenches.insert(selected, snapshot.workbench.clone());
        }
        let mut model = Self {
            snapshot,
            connection_settings,
            storage_connection_ids,
            workbenches,
            dirty_workbenches: HashSet::new(),
            saved_global_settings,
            saved_theme_mode,
            pending_connection_imports: HashMap::new(),
            pending_connection_import_secrets: Vec::new(),
            pending_connection_import_persistence: None,
        };
        model.sync_built_in_broker_connection();
        model.normalize_connection_surface();
        model
    }

    pub fn snapshot(&self) -> &AppSnapshot {
        &self.snapshot
    }

    pub(crate) fn connection_settings_for(
        &self,
        connection_id: ConnectionId,
    ) -> Option<&ConnectionSettingsSnapshot> {
        if self.snapshot.selected_connection == Some(connection_id) {
            Some(&self.snapshot.connection_settings)
        } else {
            self.connection_settings.get(&connection_id)
        }
    }

    pub(crate) fn drain_workbench_persistence_commands(
        &mut self,
    ) -> Vec<crate::HistoryPersistenceCommand> {
        let dirty: Vec<_> = self.dirty_workbenches.drain().collect();
        dirty
            .into_iter()
            .filter_map(|connection_id| {
                let workbench = self.workbench_for_connection(connection_id)?.clone();
                Some(crate::HistoryPersistenceCommand::ReplaceWorkbench {
                    connection_id: self.storage_connection_id(connection_id),
                    workbench,
                })
            })
            .collect()
    }

    pub(crate) fn drain_connection_import_persistence(
        &mut self,
    ) -> Option<(Vec<StorageConnectionConfig>, Vec<StorageImportedSecret>)> {
        self.pending_connection_import_persistence.take()
    }

    pub(crate) fn mqtt_commands_for_app_command(
        &self,
        command: &AppCommand,
    ) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
        crate::commands_for_app_command(command, &self.snapshot, &self.connection_settings)
    }

    fn select_connection_workbench(&mut self, id: ConnectionId) {
        if let Some(current) = self.snapshot.selected_connection {
            self.workbenches
                .insert(current, self.snapshot.workbench.clone());
            self.mark_workbench_dirty(current);
        }
        self.snapshot.selected_connection = Some(id);
        self.load_connection_settings(id);
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
        self.snapshot.workbench = self.workbenches.get(&id).cloned().unwrap_or_default();
    }

    pub(super) fn mark_active_workbench_dirty(&mut self) {
        if let Some(connection_id) = self.snapshot.selected_connection {
            self.mark_workbench_dirty(connection_id);
        }
    }

    pub(super) fn mark_workbench_dirty(&mut self, connection_id: ConnectionId) {
        self.dirty_workbenches.insert(connection_id);
    }

    pub(super) fn workbench_for_connection(
        &self,
        connection_id: ConnectionId,
    ) -> Option<&WorkbenchSnapshot> {
        if self.snapshot.selected_connection == Some(connection_id) {
            Some(&self.snapshot.workbench)
        } else {
            self.workbenches.get(&connection_id)
        }
    }

    pub(super) fn workbench_for_connection_mut(
        &mut self,
        connection_id: ConnectionId,
    ) -> &mut WorkbenchSnapshot {
        if self.snapshot.selected_connection == Some(connection_id) {
            &mut self.snapshot.workbench
        } else {
            self.workbenches.entry(connection_id).or_default()
        }
    }

    pub fn apply_command(&mut self, command: AppCommand) {
        if self.apply_migration_recovery_command(&command)
            || self.apply_scripting_command(&command)
            || self.apply_broker_command(&command)
            || self.apply_plugin_command(&command)
        {
            return;
        }

        let dirty_active_workbench = command_mutates_active_workbench(&command);
        match command {
            AppCommand::SelectWorkspace(workspace) => self.snapshot.active_workspace = workspace,
            AppCommand::SetThemeMode(mode) => self.set_theme_mode(mode),
            AppCommand::SearchConnections(filter) => self.snapshot.connection_filter = filter,
            AppCommand::SelectConnection(id) => self.select_connection_workbench(id),
            AppCommand::MoveConnection {
                connection_id,
                target_connection_id,
                after,
            } => self.move_connection(connection_id, target_connection_id, after),
            AppCommand::OpenConnectionLauncher => {
                self.open_default_connection_surface();
            }
            AppCommand::OpenConnectionWorkbench(id) => self.select_connection_workbench(id),
            AppCommand::Connect(id) => self.connect(id),
            AppCommand::OpenConnectionSettings(id) | AppCommand::EditConnection(id) => {
                if crate::is_built_in_broker_connection(id) {
                    self.select_connection_workbench(id);
                    self.push_diagnostic(Diagnostic::warning(
                        "Correo Broker connection is managed automatically.",
                    ));
                    return;
                }
                self.select_connection_workbench(id);
                self.load_connection_settings(id);
                self.snapshot.connection_settings_overlay = Some(id);
            }
            AppCommand::Reconnect(id) => self.record_action(id, "Reconnect requested"),
            AppCommand::Disconnect(id) => self.disconnect(id),
            AppCommand::DuplicateConnection(id) => self.record_action(id, "Duplicate requested"),
            AppCommand::AddConnection => self.add_connection(),
            AppCommand::ImportConnections => self.import_connections(),
            AppCommand::ExportConnections => self.open_connection_export(),
            AppCommand::ChooseConnectionImportFile(path) => {
                self.choose_connection_import_file(&path)
            }
            AppCommand::SubmitConnectionImportPassword(password) => {
                self.submit_connection_import_password(&password)
            }
            AppCommand::ClearConnectionImportError => self.clear_connection_import_error(),
            AppCommand::SelectConnectionImportRow { row_id, selected } => {
                self.select_connection_import_row(&row_id, selected);
            }
            AppCommand::StartConnectionImport => self.start_connection_import(),
            AppCommand::SelectConnectionExportRow { row_id, selected } => {
                self.select_connection_export_row(&row_id, selected);
            }
            AppCommand::SetConnectionExportEncrypted(encrypted) => {
                self.set_connection_export_encrypted(encrypted);
            }
            AppCommand::UpdateConnectionExportPath(path) => {
                self.update_connection_export_path(path)
            }
            AppCommand::StartConnectionExport { password } => {
                self.start_connection_export(&password)
            }
            AppCommand::ImportMessages => self.import_messages(),
            AppCommand::ImportMessagesFromPath(path) => self.import_messages_from_path(&path),
            AppCommand::ExportMessages => self.export_messages(),
            AppCommand::ExportPublishHistoryMessage(topic) => {
                self.export_publish_history_message(topic)
            }
            AppCommand::ExportIncomingMessage(id) => self.export_incoming_message(id),
            AppCommand::ExportPublishHistoryMessageToPath { message_id, path } => {
                self.export_publish_history_message_to_path(message_id, &path)
            }
            AppCommand::ExportIncomingMessageToPath { message_id, path } => {
                self.export_incoming_message_to_path(message_id, &path)
            }
            AppCommand::CopyPublishHistoryMessageToPublishForm(id) => {
                self.copy_publish_history_message_to_publish_form(id);
            }
            AppCommand::CopyIncomingMessageToPublishForm(id) => {
                self.copy_incoming_message_to_publish_form(id);
            }
            AppCommand::RemovePublishHistoryMessage(id) => self.remove_publish_history_message(id),
            AppCommand::RemoveIncomingMessage(id) => self.remove_incoming_message(id),
            AppCommand::ClearPublishHistory => self.clear_publish_history(),
            AppCommand::ClearIncomingMessages => self.clear_incoming_messages(),
            AppCommand::SelectWorkbenchTab(tab) => self.snapshot.workbench.narrow_tab = tab,
            AppCommand::UpdatePublishTopic(topic) => self.update_publish_topic(topic),
            AppCommand::UpdatePublishPayload(payload) => self.update_publish_payload(payload),
            AppCommand::UpdatePublishQos(qos) => self.update_publish_qos(qos),
            AppCommand::SetPublishRetained(retained) => {
                self.snapshot.workbench.publish.retained = retained;
            }
            AppCommand::SearchPublishHistory(filter) => {
                self.snapshot.workbench.publish.history_filter = filter;
            }
            AppCommand::SelectPublishHistoryMessage(id) => {
                self.snapshot.workbench.publish.selected_history_id = Some(id);
            }
            AppCommand::Publish => self.publish_from_snapshot(),
            AppCommand::UpdateSubscribeTopic(topic) => self.update_subscribe_topic(topic),
            AppCommand::UpdateSubscribeQos(qos) => self.update_subscribe_qos(qos),
            AppCommand::Subscribe => self.subscribe_from_snapshot(),
            AppCommand::Unsubscribe(topic) => self.unsubscribe(&topic),
            AppCommand::UnsubscribeAll => self.request_unsubscribe_all(),
            AppCommand::CancelUnsubscribeAll => self.cancel_unsubscribe_all(),
            AppCommand::ConfirmUnsubscribeAll => self.confirm_unsubscribe_all(),
            AppCommand::SetSubscriptionMessagesVisible {
                topic_filter,
                visible,
            } => self.set_subscription_messages_visible(&topic_filter, visible),
            AppCommand::SetAllSubscriptionMessagesVisible(visible) => {
                self.set_all_subscription_messages_visible(visible);
            }
            AppCommand::SelectSubscription {
                topic_filter,
                extend,
                toggle,
            } => self.select_subscription(&topic_filter, extend, toggle),
            AppCommand::SearchMessages(filter) => {
                self.snapshot.workbench.subscribe.message_filter = filter;
            }
            AppCommand::SelectMessage(id) => self.snapshot.workbench.selected_message_id = Some(id),
            AppCommand::SelectInspectorTab(tab) => self.snapshot.workbench.inspector_tab = tab,
            AppCommand::SelectDetailTransform(plugin_id) => self.select_detail_transform(plugin_id),
            AppCommand::SelectDetailFormatter(plugin_id) => self.select_detail_formatter(plugin_id),
            AppCommand::RefreshMessageDetail => {}
            AppCommand::SelectConnectionSettingsTab(tab) => {
                self.snapshot.connection_settings.selected_tab = tab;
            }
            AppCommand::UpdateConnectionSetting { field, value } => {
                self.update_connection_setting(field, value);
            }
            AppCommand::UpdateConnectionSecret { field, value } => {
                self.update_connection_secret(field, value);
            }
            AppCommand::SetConnectionSettingFlag { flag, enabled } => {
                self.set_connection_setting_flag(flag, enabled);
            }
            AppCommand::GenerateClientId => self.generate_client_id(),
            AppCommand::SetLwtEnabled(enabled) => {
                self.snapshot.connection_settings.lwt_enabled = enabled;
                self.snapshot.connection_settings.dirty = true;
                self.refresh_connection_settings_validation();
            }
            AppCommand::SaveConnectionSettings => self.save_connection_settings(),
            AppCommand::DiscardConnectionSettings => self.discard_connection_settings(),
            AppCommand::OpenConnectionPlugins(connection_id) => {
                self.open_connection_plugins(connection_id)
            }
            AppCommand::SaveConnectionPlugins => self.save_connection_plugins(),
            AppCommand::CloseConnectionPlugins => self.close_connection_plugins(),
            AppCommand::SelectConnectionPluginWorkflow(index) => {
                self.snapshot.connection_settings.selected_plugin_workflow = Some(index)
            }
            AppCommand::AddConnectionPluginWorkflow { plugin_id } => {
                self.add_connection_plugin_workflow(plugin_id)
            }
            AppCommand::RemoveConnectionPluginWorkflow { index } => {
                self.remove_connection_plugin_workflow(index)
            }
            AppCommand::SetConnectionPluginWorkflowEnabled { index, enabled } => {
                self.set_connection_plugin_workflow_enabled(index, enabled)
            }
            AppCommand::MoveConnectionPluginWorkflow {
                index,
                target_index,
                after,
            } => self.move_connection_plugin_workflow(index, target_index, after),
            AppCommand::SetConnectionPluginWorkflowDirection { index, direction } => {
                self.set_connection_plugin_workflow_direction(index, direction)
            }
            AppCommand::UpdateConnectionPluginWorkflowField {
                index,
                field,
                value,
            } => self.update_connection_plugin_workflow_field(index, field, value),
            AppCommand::RequestDeleteConnection => self.request_delete_connection(),
            AppCommand::CancelDeleteConnection => {
                self.snapshot.connection_settings.delete_confirmation_open = false;
            }
            AppCommand::ConfirmDeleteConnection => {
                self.delete_selected_connection();
            }
            AppCommand::SelectTransferSection(section) => {
                self.snapshot.transfer.active_section = section;
            }
            AppCommand::SelectTransferStep(step) => self.select_import_step(step),
            AppCommand::SelectGlobalSettingsSection(section) => {
                self.select_global_settings_section(section);
            }
            AppCommand::UpdateGlobalSetting { field, value } => {
                self.update_global_setting(field, value);
            }
            AppCommand::SetGlobalSettingFlag { flag, enabled } => {
                self.set_global_setting_flag(flag, enabled);
            }
            AppCommand::AddPluginRepository => self.add_plugin_repository(),
            AppCommand::UpdatePluginRepository { index, url } => {
                self.update_plugin_repository(index, url);
            }
            AppCommand::RemovePluginRepository { index } => {
                self.remove_plugin_repository(index);
            }
            AppCommand::SaveGlobalSettings => self.save_global_settings(),
            AppCommand::DiscardGlobalSettings => self.discard_global_settings(),
            AppCommand::Mqtt(command) => self.apply_mqtt_command(command),
            AppCommand::Shutdown => {}
            _ => unreachable!("handled before main dispatch"),
        }
        if dirty_active_workbench {
            self.mark_active_workbench_dirty();
        }
    }
}

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
                    .insert(connection_id, settings.clone());
                self.snapshot.connection_settings = settings;
            }
            AppEvent::GlobalSettingsLoaded { settings } => self.load_global_settings(settings),
            AppEvent::ThemeModeChanged { mode } => self.snapshot.theme_mode = mode,
            AppEvent::MigrationApplied {
                state,
                completion,
                diagnostics,
            } => self.apply_migrated_startup_state(*state, completion, diagnostics),
            AppEvent::DiagnosticRaised(diagnostic) => self.push_diagnostic(diagnostic),
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
