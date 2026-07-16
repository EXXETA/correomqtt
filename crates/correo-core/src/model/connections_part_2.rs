impl AppModel {
    pub(super) fn discard_connection_settings(&mut self) {
        if let Some(id) = self.snapshot.selected_connection {
            if let Some(settings) = self.connection_settings.get(&id) {
                self.snapshot.connection_settings = settings.clone();
            } else {
                self.snapshot.connection_settings.dirty = false;
            }
            self.snapshot.connection_settings_overlay = None;
            self.push_diagnostic(Diagnostic::info("Connection settings discarded."));
            return;
        }

        self.snapshot.connection_settings = ConnectionSettingsSnapshot::default();
        self.snapshot.connection_settings_overlay = None;
        self.open_default_connection_surface();
        self.push_diagnostic(Diagnostic::info("New connection draft discarded."));
    }

    pub(super) fn request_delete_connection(&mut self) {
        let Some(id) = self.snapshot.selected_connection else {
            self.snapshot.connection_settings.delete_confirmation_open = false;
            return;
        };
        if crate::is_built_in_broker_connection(id) {
            self.snapshot.connection_settings.delete_confirmation_open = false;
            self.push_diagnostic(Diagnostic::warning(
                "Correo Broker connection can not be deleted.",
            ));
            return;
        }
        if self.snapshot.connection_settings_overlay != Some(id)
            && self.snapshot.connection_surface != crate::ConnectionSurface::Settings
        {
            self.load_connection_settings(id);
            if !self.connection_settings.contains_key(&id) {
                if let Some(connection) = self.snapshot.connections.iter().find(|row| row.id == id)
                {
                    self.snapshot.connection_settings.profile_name = connection.name.clone();
                }
            }
        }
        self.snapshot.connection_settings.delete_confirmation_open = true;
    }

    pub(super) fn delete_selected_connection(&mut self) {
        let Some(id) = self.snapshot.selected_connection else {
            self.snapshot.connection_settings.delete_confirmation_open = false;
            return;
        };
        let Some(index) = self.connection_index(id) else {
            self.snapshot.connection_settings.delete_confirmation_open = false;
            self.snapshot.connection_settings_overlay = None;
            self.open_default_connection_surface();
            return;
        };

        let name = self.snapshot.connections.remove(index).name;
        self.connection_settings.remove(&id);
        self.storage_connection_ids.remove(&id);
        self.workbenches.remove(&id);
        self.dirty_workbenches.remove(&id);
        if self.snapshot.active_connection == Some(id) {
            self.snapshot.active_connection = None;
        }

        self.snapshot.connection_count = self.snapshot.connections.len();
        self.snapshot.connection_settings_overlay = None;
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
        self.snapshot.selected_connection = self
            .snapshot
            .connections
            .first()
            .map(|connection| connection.id);
        self.snapshot.workbench = self
            .snapshot
            .selected_connection
            .and_then(|selected| self.workbenches.get(&selected).cloned())
            .unwrap_or_default();
        self.snapshot.connection_settings = self
            .snapshot
            .selected_connection
            .and_then(|selected| self.connection_settings.get(&selected).cloned())
            .unwrap_or_default();
        self.push_diagnostic(Diagnostic::info(format!("Deleted connection {name}.")));
    }

    pub(super) fn move_connection(
        &mut self,
        connection_id: ConnectionId,
        target_connection_id: ConnectionId,
        after: bool,
    ) {
        if connection_id == target_connection_id {
            return;
        }
        if crate::is_built_in_broker_connection(connection_id)
            || crate::is_built_in_broker_connection(target_connection_id)
        {
            return;
        }

        let Some(from_index) = self.connection_index(connection_id) else {
            return;
        };
        let Some(target_index) = self.connection_index(target_connection_id) else {
            return;
        };

        let connection = self.snapshot.connections.remove(from_index);
        let mut insert_index = if from_index < target_index {
            target_index.saturating_sub(1)
        } else {
            target_index
        };
        if after {
            insert_index += 1;
        }
        let insert_index = insert_index.min(self.snapshot.connections.len());
        self.snapshot.connections.insert(insert_index, connection);
    }

    pub(super) fn update_connection_setting(
        &mut self,
        field: ConnectionSettingField,
        value: String,
    ) {
        let settings = &mut self.snapshot.connection_settings;
        match field {
            ConnectionSettingField::ProfileName => settings.profile_name = value,
            ConnectionSettingField::Host => settings.host = value,
            ConnectionSettingField::Port => settings.port = value,
            ConnectionSettingField::MqttVersion => settings.mqtt_version = value,
            ConnectionSettingField::ClientId => settings.client_id = value,
            ConnectionSettingField::Username => settings.username = value,
            ConnectionSettingField::TlsMode => settings.tls_mode = value,
            ConnectionSettingField::TlsStore => settings.tls_store = value,
            ConnectionSettingField::ProxyMode => settings.proxy_mode = value,
            ConnectionSettingField::SshHost => settings.ssh_host = value,
            ConnectionSettingField::SshPort => settings.ssh_port = value,
            ConnectionSettingField::LocalMqttPort => settings.local_mqtt_port = value,
            ConnectionSettingField::AuthMode => settings.auth_mode = value,
            ConnectionSettingField::AuthUsername => settings.auth_username = value,
            ConnectionSettingField::SshKeyFile => settings.ssh_key_file = value,
            ConnectionSettingField::LwtTopic => settings.lwt_topic = value,
            ConnectionSettingField::LwtPayload => settings.lwt_payload = value,
        }
        settings.dirty = true;
        refresh_connection_settings_validation(settings);
    }

    pub(super) fn update_connection_secret(
        &mut self,
        field: ConnectionSecretField,
        value: SecretInput,
    ) {
        let settings = &mut self.snapshot.connection_settings;
        match field {
            ConnectionSecretField::MqttPassword => settings.password = value,
            ConnectionSecretField::TlsKeystorePassword => settings.tls_keystore_password = value,
            ConnectionSecretField::SshPassword => settings.ssh_password = value,
        }
        settings.dirty = true;
    }

    pub(super) fn set_connection_setting_flag(
        &mut self,
        flag: ConnectionSettingFlag,
        enabled: bool,
    ) {
        let settings = &mut self.snapshot.connection_settings;
        match flag {
            ConnectionSettingFlag::CleanSession => settings.clean_session = enabled,
            ConnectionSettingFlag::TlsHostVerification => settings.tls_host_verification = enabled,
            ConnectionSettingFlag::LwtRetained => settings.lwt_retained = enabled,
        }
        settings.dirty = true;
        refresh_connection_settings_validation(settings);
    }

    pub(super) fn generate_client_id(&mut self) {
        self.snapshot.connection_settings.client_id =
            format!("correomqtt-{}", uuid::Uuid::new_v4().simple());
        self.snapshot.connection_settings.dirty = true;
        refresh_connection_settings_validation(&mut self.snapshot.connection_settings);
    }

    pub(super) fn refresh_connection_settings_validation(&mut self) {
        refresh_connection_settings_validation(&mut self.snapshot.connection_settings);
    }

    pub(super) fn record_action(&mut self, id: ConnectionId, action: &'static str) {
        let name = self
            .snapshot
            .connections
            .iter()
            .find(|connection| connection.id == id)
            .map(|connection| connection.name.clone())
            .unwrap_or_else(|| "Unknown connection".to_owned());
        self.push_diagnostic(Diagnostic::info(format!("{action}: {name}.")));
    }

    pub(super) fn connection_index(&self, id: ConnectionId) -> Option<usize> {
        self.snapshot
            .connections
            .iter()
            .position(|connection| connection.id == id)
    }

    fn ensure_selected_connection(&mut self) {
        if self
            .snapshot
            .selected_connection
            .is_some_and(|id| self.connection_index(id).is_some())
        {
            return;
        }

        self.snapshot.selected_connection = self
            .snapshot
            .connections
            .first()
            .map(|connection| connection.id);
        if let Some(id) = self.snapshot.selected_connection {
            self.load_connection_settings(id);
        }
    }

    pub(super) fn update_connection_state(
        &mut self,
        id: ConnectionId,
        state: ConnectionState,
        disabled_reason: Option<ConnectDisabledReason>,
        last_activity: String,
    ) {
        if let Some(index) = self.connection_index(id) {
            let connection = &mut self.snapshot.connections[index];
            connection.state = state;
            connection.disabled_reason = disabled_reason;
            connection.last_activity = last_activity;
        }
    }

    pub(super) fn load_connection_settings(&mut self, id: ConnectionId) {
        if crate::is_built_in_broker_connection(id) {
            self.sync_built_in_broker_connection();
        }
        if let Some(settings) = self.connection_settings.get(&id) {
            self.snapshot.connection_settings = settings.clone();
        }
    }

    fn ensure_connection_plugin_workflows(&mut self) {
        let available_ids = self
            .snapshot
            .plugins
            .plugins
            .iter()
            .filter(|plugin| plugin.enabled && plugin.status == PluginStatus::Active)
            .map(|plugin| (plugin.id.clone(), plugin.name.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        for workflow in &mut self.snapshot.connection_settings.plugin_workflows {
            workflow.available = available_ids.contains_key(&workflow.plugin_id);
            if let Some(name) = available_ids.get(&workflow.plugin_id) {
                workflow.plugin_name = name.clone();
            }
            workflow.status = match (workflow.available, workflow.enabled) {
                (true, true) => ConnectionPluginWorkflowStatus::Ready,
                (_, false) => ConnectionPluginWorkflowStatus::Disabled,
                (false, true) => ConnectionPluginWorkflowStatus::MissingPlugin,
            };
        }
    }

    fn connection_workflow_plugin(
        &self,
        plugin_id: &str,
    ) -> Option<(String, ConnectionPluginWorkflowKind)> {
        let kind = match plugin_id {
            CONTAINS_STRING_ID | XML_XSD_ID => ConnectionPluginWorkflowKind::Validator,
            BASE64_ID | SAVE_ID | ZIP_ID => ConnectionPluginWorkflowKind::Manipulator,
            _ => return None,
        };
        let plugin = self.snapshot.plugins.plugins.iter().find(|plugin| {
            plugin.id == plugin_id && plugin.enabled && plugin.status == PluginStatus::Active
        })?;
        Some((plugin.name.clone(), kind))
    }

    fn update_connection_summary(
        &mut self,
        id: ConnectionId,
        settings: &ConnectionSettingsSnapshot,
    ) {
        if let Some(index) = self.connection_index(id) {
            let state = self.snapshot.connections[index].state;
            let disabled_reason = if settings.host.trim().is_empty() {
                Some(ConnectDisabledReason::MissingHost)
            } else {
                self.snapshot.connections[index].disabled_reason
            };
            self.snapshot.connections[index] = ConnectionSummary {
                state,
                disabled_reason,
                recent_subscriptions: self.snapshot.connections[index].recent_subscriptions,
                recent_messages: self.snapshot.connections[index].recent_messages,
                last_activity: self.snapshot.connections[index].last_activity.clone(),
                ..connection_summary(id, settings)
            };
        }
    }
}

impl AppModel {
    pub(super) fn sync_built_in_broker_connection(&mut self) {
        let id = crate::built_in_broker_connection_id();
        let mut summary = crate::built_in_broker_connection_summary(&self.snapshot.built_in_broker);
        if let Some(existing) = self
            .snapshot
            .connections
            .iter()
            .find(|connection| connection.id == id)
        {
            summary.state = existing.state;
            if existing.state != ConnectionState::Disconnected {
                summary.disabled_reason = existing.disabled_reason;
            }
            summary.recent_subscriptions = existing.recent_subscriptions;
            summary.recent_messages = existing.recent_messages;
            summary.last_activity = existing.last_activity.clone();
        }
        self.snapshot
            .connections
            .retain(|connection| connection.id != id);
        self.snapshot.connections.insert(0, summary);
        self.connection_settings.insert(
            id,
            crate::built_in_broker_connection_settings(&self.snapshot.built_in_broker),
        );
        self.snapshot.connection_count = self.snapshot.connections.len();
        if self.snapshot.selected_connection == Some(id) {
            if let Some(settings) = self.connection_settings.get(&id) {
                self.snapshot.connection_settings = settings.clone();
            }
        }
    }
}

fn default_connection_plugin_workflow(
    plugin_id: &str,
    plugin_name: String,
    kind: ConnectionPluginWorkflowKind,
) -> ConnectionPluginWorkflow {
    let config = match plugin_id {
        CONTAINS_STRING_ID => serde_json::json!({ "rules": [] }),
        XML_XSD_ID => serde_json::json!({ "xsd_path": "" }),
        SAVE_ID => serde_json::json!({ "folder": "" }),
        BASE64_ID | ZIP_ID => serde_json::json!({}),
        _ => serde_json::json!({}),
    };
    ConnectionPluginWorkflow {
        plugin_id: plugin_id.to_owned(),
        plugin_name,
        enabled: true,
        kind,
        direction: ConnectionPluginDirection::Incoming,
        topic_filter: "#".to_owned(),
        config,
        available: true,
        status: ConnectionPluginWorkflowStatus::Ready,
        message: String::new(),
    }
}

fn new_connection_settings() -> ConnectionSettingsSnapshot {
    let mut settings = ConnectionSettingsSnapshot {
        selected_tab: ConnectionSettingsTab::Mqtt,
        internal_id: "Generated on save".to_owned(),
        profile_name: "New connection".to_owned(),
        port: "1883".to_owned(),
        mqtt_version: "MQTT v5".to_owned(),
        clean_session: true,
        password_status: "No MQTT password configured".to_owned(),
        tls_mode: "No TLS/SSL".to_owned(),
        tls_password_status: "No SSL password configured".to_owned(),
        tls_host_verification: true,
        proxy_mode: "No proxy/tunnel".to_owned(),
        ssh_port: "22".to_owned(),
        local_mqtt_port: "1883".to_owned(),
        auth_mode: "No Auth".to_owned(),
        ssh_password_status: "No SSH password configured".to_owned(),
        lwt_qos: QosLevel::One,
        lwt_retained: false,
        dirty: true,
        keyring_state: KeyringState::Available,
        ..ConnectionSettingsSnapshot::default()
    };
    refresh_connection_settings_validation(&mut settings);
    settings
}

fn refresh_connection_settings_validation(settings: &mut ConnectionSettingsSnapshot) {
    let mut errors = Vec::new();
    if settings.profile_name.trim().is_empty() {
        errors.push("Name is required".to_owned());
    }
    if settings.host.trim().is_empty() {
        errors.push("Host is required".to_owned());
    }
    match settings.port.trim().parse::<u16>() {
        Ok(0) | Err(_) => errors.push("Port must be between 1 and 65535".to_owned()),
        Ok(_) => {}
    }
    if settings.tls_mode == "Keystore" && settings.tls_store.trim().is_empty() {
        errors.push("SSL keystore is required when TLS/SSL uses Keystore".to_owned());
    }
    if settings.proxy_mode == "SSH" {
        if settings.ssh_host.trim().is_empty() {
            errors.push("SSH host is required".to_owned());
        }
        validate_optional_port(&settings.ssh_port, "SSH port", true, &mut errors);
        validate_optional_port(
            &settings.local_mqtt_port,
            "Local MQTT port",
            false,
            &mut errors,
        );
        if settings.auth_mode != "No Auth" && settings.auth_username.trim().is_empty() {
            errors.push("SSH username is required".to_owned());
        }
        if settings.auth_mode == "Keyfile" && settings.ssh_key_file.trim().is_empty() {
            errors.push("SSH key file is required".to_owned());
        }
    }

    settings.valid = errors.is_empty();
    settings.save_disabled_reason = if settings.valid {
        "No changes to save".to_owned()
    } else {
        "Resolve validation errors before saving".to_owned()
    };
    settings.validation_errors = errors;
}

