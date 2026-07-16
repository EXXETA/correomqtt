use correo_mqtt::ConnectionId;
use correo_storage::current::{ConnectionConfig as StorageConnectionConfig, ImportedSecret};

use crate::{
    AppModel, ConnectDisabledReason, ConnectionBadge, ConnectionPluginDirection,
    ConnectionPluginWorkflow, ConnectionPluginWorkflowField, ConnectionPluginWorkflowKind,
    ConnectionPluginWorkflowStatus, ConnectionSecretField, ConnectionSettingField,
    ConnectionSettingFlag, ConnectionSettingsSnapshot, ConnectionSettingsTab, ConnectionState,
    ConnectionSummary, Diagnostic, KeyringState, PluginStatus, QosLevel, SecretInput,
};

const CONTAINS_STRING_ID: &str = "org.correomqtt.plugins.contains-string-validator";
const XML_XSD_ID: &str = "org.correomqtt.plugins.xml-xsd-validator";
const BASE64_ID: &str = "org.correomqtt.plugins.base64";
const SAVE_ID: &str = "org.correomqtt.plugins.save-manipulator";
const ZIP_ID: &str = "org.correomqtt.plugins.zip-manipulator";

impl AppModel {
    pub(super) fn normalize_connection_surface(&mut self) {
        self.sync_built_in_broker_connection();
        self.ensure_selected_connection();
        if let Some(id) = self.snapshot.selected_connection {
            self.load_connection_settings(id);
        }
        if self.snapshot.connection_surface == crate::ConnectionSurface::Launcher {
            self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
        }
    }

    pub(super) fn open_default_connection_surface(&mut self) {
        self.snapshot.active_workspace = crate::Workspace::Connections;
        self.ensure_selected_connection();
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
    }

    pub(super) fn add_connection(&mut self) {
        self.snapshot.active_workspace = crate::Workspace::Connections;
        self.snapshot.selected_connection = None;
        self.snapshot.connection_surface = crate::ConnectionSurface::Settings;
        self.snapshot.connection_settings_overlay = None;
        self.snapshot.connection_plugins_overlay = None;
        self.snapshot.connection_settings = new_connection_settings();
        self.push_diagnostic(Diagnostic::info("New connection draft opened."));
    }

    pub(super) fn add_imported_connection(
        &mut self,
        connection: StorageConnectionConfig,
        secrets: &[ImportedSecret],
    ) -> ConnectionId {
        let id = ConnectionId::new();
        let settings = crate::bootstrap::settings_snapshot(&connection, &[], Some(secrets));
        self.connection_settings.insert(id, settings.clone());
        self.storage_connection_ids
            .insert(id, connection.id.clone());
        self.snapshot
            .connections
            .push(connection_summary(id, &settings));
        self.snapshot.connection_count = self.snapshot.connections.len();
        id
    }

    pub(super) fn connect(&mut self, id: ConnectionId) {
        let Some(index) = self.connection_index(id) else {
            return;
        };

        if !self.snapshot.connections[index].can_connect() {
            let reason = self.snapshot.connections[index]
                .disabled_reason
                .unwrap_or(ConnectDisabledReason::Busy)
                .label();
            self.push_diagnostic(Diagnostic::warning(reason));
            return;
        }

        let name = self.snapshot.connections[index].name.clone();
        self.update_connection_state(
            id,
            ConnectionState::Connecting,
            Some(ConnectDisabledReason::Busy),
            "connect command queued".to_owned(),
        );
        self.snapshot.selected_connection = Some(id);
        self.load_connection_settings(id);
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
        self.push_diagnostic(Diagnostic::info(format!("Connect requested for {name}.")));
    }

    pub(super) fn disconnect(&mut self, id: ConnectionId) {
        let Some(index) = self.connection_index(id) else {
            return;
        };
        let name = {
            let connection = &mut self.snapshot.connections[index];
            connection.state = ConnectionState::Disconnected;
            connection.disabled_reason = None;
            connection.name.clone()
        };
        self.snapshot.active_connection = None;
        self.push_diagnostic(Diagnostic::info(format!(
            "{name} disconnect command queued."
        )));
    }

    pub(super) fn save_connection_settings(&mut self) {
        if self
            .snapshot
            .selected_connection
            .is_some_and(crate::is_built_in_broker_connection)
        {
            self.push_diagnostic(Diagnostic::warning(
                "Correo Broker connection is managed automatically.",
            ));
            return;
        }
        refresh_connection_settings_validation(&mut self.snapshot.connection_settings);
        if !self.snapshot.connection_settings.valid {
            for error in self.snapshot.connection_settings.validation_errors.clone() {
                self.push_diagnostic(Diagnostic::warning(error));
            }
            return;
        }
        if !self.snapshot.connection_settings.dirty {
            let reason = self
                .snapshot
                .connection_settings
                .save_disabled_reason
                .clone();
            self.push_diagnostic(Diagnostic::warning(reason));
            return;
        }

        let mut settings = self.snapshot.connection_settings.clone();
        settings.dirty = false;
        settings.delete_confirmation_open = false;
        settings.save_disabled_reason = "No changes to save".to_owned();

        if let Some(id) = self.snapshot.selected_connection {
            if settings.internal_id.trim().is_empty() {
                settings.internal_id = id.to_string();
            }
            self.connection_settings.insert(id, settings.clone());
            self.snapshot.connection_settings = settings.clone();
            self.update_connection_summary(id, &settings);
            self.snapshot.connection_settings_overlay = None;
            self.push_diagnostic(Diagnostic::info("Connection settings save command queued."));
            return;
        }

        let id = ConnectionId::new();
        settings.internal_id = id.to_string();
        self.connection_settings.insert(id, settings.clone());
        self.storage_connection_ids.insert(id, id.to_string());
        self.snapshot
            .connections
            .push(connection_summary(id, &settings));
        self.snapshot.connection_count = self.snapshot.connections.len();
        self.snapshot.selected_connection = Some(id);
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
        self.snapshot.connection_settings = settings;
        self.push_diagnostic(Diagnostic::info("New connection profile saved."));
    }

    pub(super) fn open_connection_plugins(&mut self, id: ConnectionId) {
        self.snapshot.selected_connection = Some(id);
        self.snapshot.connection_plugins_overlay = Some(id);
        self.load_connection_settings(id);
        self.ensure_connection_plugin_workflows();
        self.snapshot
            .connection_settings
            .plugin_workflow_dialog_open = true;
    }

    pub(super) fn save_connection_plugins(&mut self) {
        self.snapshot.connection_settings.dirty = true;
        self.snapshot
            .connection_settings
            .plugin_workflow_dialog_open = false;
        self.snapshot.connection_plugins_overlay = None;
        self.save_connection_settings();
    }

    pub(super) fn close_connection_plugins(&mut self) {
        if let Some(id) = self.snapshot.selected_connection {
            if let Some(settings) = self.connection_settings.get(&id) {
                self.snapshot.connection_settings.plugin_workflows =
                    settings.plugin_workflows.clone();
            }
        }
        self.snapshot
            .connection_settings
            .plugin_workflow_dialog_open = false;
        self.snapshot.connection_plugins_overlay = None;
    }

    pub(super) fn set_connection_plugin_workflow_enabled(&mut self, index: usize, enabled: bool) {
        if let Some(workflow) = self
            .snapshot
            .connection_settings
            .plugin_workflows
            .get_mut(index)
        {
            workflow.enabled = enabled;
            workflow.status = if enabled {
                if workflow.available {
                    ConnectionPluginWorkflowStatus::Ready
                } else {
                    ConnectionPluginWorkflowStatus::MissingPlugin
                }
            } else {
                ConnectionPluginWorkflowStatus::Disabled
            };
            self.snapshot.connection_settings.dirty = true;
        }
    }

    pub(super) fn add_connection_plugin_workflow(&mut self, plugin_id: String) {
        let Some((plugin_name, kind)) = self.connection_workflow_plugin(&plugin_id) else {
            return;
        };
        self.snapshot.connection_settings.plugin_workflows.push(
            default_connection_plugin_workflow(&plugin_id, plugin_name, kind),
        );
        self.snapshot.connection_settings.selected_plugin_workflow =
            Some(self.snapshot.connection_settings.plugin_workflows.len() - 1);
        self.snapshot.connection_settings.dirty = true;
    }

    pub(super) fn remove_connection_plugin_workflow(&mut self, index: usize) {
        if index >= self.snapshot.connection_settings.plugin_workflows.len() {
            return;
        }
        self.snapshot
            .connection_settings
            .plugin_workflows
            .remove(index);
        let len = self.snapshot.connection_settings.plugin_workflows.len();
        self.snapshot.connection_settings.selected_plugin_workflow = if len == 0 {
            None
        } else {
            Some(index.min(len - 1))
        };
        self.snapshot.connection_settings.dirty = true;
    }

    pub(super) fn move_connection_plugin_workflow(
        &mut self,
        index: usize,
        target_index: usize,
        after: bool,
    ) {
        let len = self.snapshot.connection_settings.plugin_workflows.len();
        if index >= len || target_index >= len || index == target_index {
            return;
        }
        let item = self
            .snapshot
            .connection_settings
            .plugin_workflows
            .remove(index);
        let mut insert_at = target_index;
        if index < target_index {
            insert_at = insert_at.saturating_sub(1);
        }
        if after {
            insert_at = insert_at.saturating_add(1);
        }
        insert_at = insert_at.min(self.snapshot.connection_settings.plugin_workflows.len());
        self.snapshot
            .connection_settings
            .plugin_workflows
            .insert(insert_at, item);
        self.snapshot.connection_settings.selected_plugin_workflow = Some(insert_at);
        self.snapshot.connection_settings.dirty = true;
    }

    pub(super) fn set_connection_plugin_workflow_direction(
        &mut self,
        index: usize,
        direction: ConnectionPluginDirection,
    ) {
        if let Some(workflow) = self
            .snapshot
            .connection_settings
            .plugin_workflows
            .get_mut(index)
        {
            workflow.direction = direction;
            self.snapshot.connection_settings.dirty = true;
        }
    }

    pub(super) fn update_connection_plugin_workflow_field(
        &mut self,
        index: usize,
        field: ConnectionPluginWorkflowField,
        value: String,
    ) {
        let Some(workflow) = self
            .snapshot
            .connection_settings
            .plugin_workflows
            .get_mut(index)
        else {
            return;
        };
        match field {
            ConnectionPluginWorkflowField::TopicFilter => workflow.topic_filter = value,
            ConnectionPluginWorkflowField::ContainsStrings => {
                workflow.config["rules"] = serde_json::Value::Array(
                    value
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(|line| {
                            let line = line.trim();
                            if let Some(regex) = line.strip_prefix("regex:") {
                                serde_json::json!({ "text": regex.trim(), "regex": true })
                            } else {
                                serde_json::json!({ "text": line, "regex": false })
                            }
                        })
                        .collect(),
                );
            }
            ConnectionPluginWorkflowField::XsdPath => workflow.config["xsd_path"] = value.into(),
            ConnectionPluginWorkflowField::SaveFolder => workflow.config["folder"] = value.into(),
            ConnectionPluginWorkflowField::FeatureEnabled => {
                workflow.config["feature_enabled"] = serde_json::Value::Bool(value == "true")
            }
        }
        self.snapshot.connection_settings.dirty = true;
    }

}

