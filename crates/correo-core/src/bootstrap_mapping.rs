fn workbench_from_history(history: &ConnectionHistorySnapshot) -> WorkbenchSnapshot {
    let mut workbench = WorkbenchSnapshot::default();
    workbench.publish.topic_history = history.publish_topics.topics.clone();
    workbench.publish.history = history
        .publish_messages
        .messages
        .iter()
        .enumerate()
        .map(|(index, message)| {
            let payload = message.payload.clone().unwrap_or_default().into_bytes();
            let mut badges = Vec::new();
            if message.retained {
                badges.push("retained".to_owned());
            }
            PublishHistoryRow {
                id: (index as u32).saturating_add(1),
                topic: message.topic.clone(),
                timestamp: message
                    .date_time
                    .clone()
                    .unwrap_or_else(|| "migrated".to_owned()),
                qos: message.qos.map(qos).unwrap_or_default(),
                retained: message.retained,
                payload_preview: payload_preview(&payload),
                byte_size: payload.len(),
                payload,
                badges,
                diagnostics: Vec::new(),
            }
        })
        .collect();
    workbench.subscribe = SubscribePaneSnapshot {
        topic_history: history.subscriptions.topics.clone(),
        subscriptions: history
            .subscriptions
            .topics
            .iter()
            .map(|topic| SubscriptionRow {
                topic_filter: topic.clone(),
                qos: QosLevel::Zero,
                message_count: 0,
                active: true,
                messages_visible: true,
                selected: false,
            })
            .collect(),
        ..SubscribePaneSnapshot::default()
    };
    workbench
}

fn global_settings(settings: &Settings) -> GlobalSettingsSnapshot {
    let mut snapshot = GlobalSettingsSnapshot {
        language: settings
            .saved_locale
            .clone()
            .or_else(|| settings.current_locale.clone())
            .unwrap_or_else(|| "system".to_owned()),
        keyring_backend: normalize_keyring_backend(
            settings.keyring_identifier.as_deref().unwrap_or("os"),
        ),
        update_checks_enabled: settings.search_updates,
        last_update_check: "Not checked this session".to_owned(),
        cleanup_status: "Sensitive values remain outside the UI snapshot".to_owned(),
        search_use_regex: settings.use_regex_for_search,
        search_ignore_case: settings.use_ignore_case,
        reduce_motion: settings.reduce_motion,
        use_default_plugin_repository: settings.use_default_repo,
        install_bundled_plugins: settings.install_bundled_plugins,
        bundled_plugins_url: settings.bundled_plugins_url.clone().unwrap_or_default(),
        plugin_repositories: settings
            .plugin_repositories
            .iter()
            .map(|(id, url)| PluginRepositoryRow {
                id: id.clone(),
                url: url.clone(),
            })
            .collect(),
        plugin_states: settings
            .plugin_states
            .iter()
            .map(|(plugin_id, state)| {
                (
                    plugin_id.clone(),
                    PluginStateSnapshot {
                        enabled: state.enabled,
                    },
                )
            })
            .collect(),
        plugin_hooks: settings
            .plugin_hooks
            .iter()
            .map(|(plugin_id, hooks)| {
                (
                    plugin_id.clone(),
                    hooks.iter().map(plugin_hook_settings).collect(),
                )
            })
            .collect(),
        first_start: settings.first_start,
        config_version: settings
            .config_created_with_correo_version
            .clone()
            .unwrap_or_else(|| "unknown".to_owned()),
        ..GlobalSettingsSnapshot::default()
    };
    if let Some(geometry) = &settings.global_ui_settings {
        snapshot.window_geometry = format!(
            "{:.0}x{:.0} at {:.0},{:.0}",
            geometry.window_width,
            geometry.window_height,
            geometry.window_position_x,
            geometry.window_position_y
        );
    }
    snapshot
}

fn plugin_hook_settings(settings: &PluginHookSettings) -> PluginHookSettingsSnapshot {
    PluginHookSettingsSnapshot {
        hook: plugin_hook_kind(settings.hook),
        enabled: settings.enabled,
        target: settings.target.clone(),
        config_json: settings.config_json.clone(),
    }
}

fn plugin_hook_kind(kind: StoragePluginHookKind) -> crate::PluginHookKind {
    match kind {
        StoragePluginHookKind::IncomingTransform => crate::PluginHookKind::IncomingTransform,
        StoragePluginHookKind::OutgoingTransform => crate::PluginHookKind::OutgoingTransform,
        StoragePluginHookKind::Validator => crate::PluginHookKind::Validator,
        StoragePluginHookKind::DetailTransform => crate::PluginHookKind::DetailTransform,
        StoragePluginHookKind::DetailFormatter => crate::PluginHookKind::DetailFormatter,
    }
}

fn badges(connection: &ConnectionConfig) -> Vec<ConnectionBadge> {
    let mut badges = Vec::new();
    if connection
        .username
        .as_ref()
        .is_some_and(|name| !name.trim().is_empty())
    {
        badges.push(ConnectionBadge::Credentials);
    }
    if connection.ssl != TlsSsl::Off {
        badges.push(ConnectionBadge::Tls);
    }
    if connection.proxy != Proxy::Off {
        badges.push(ConnectionBadge::Proxy);
    }
    if connection.lwt == Lwt::On {
        badges.push(ConnectionBadge::Lwt);
    }
    badges
}

fn theme_mode(settings: Option<&ThemeSettings>) -> Option<ThemeMode> {
    settings
        .and_then(|settings| settings.active_theme.as_ref())
        .and_then(|theme| theme.name.as_deref())
        .map(ThemeMode::parse)
}

fn normalize_publish_history_ids(workbench: &mut WorkbenchSnapshot) {
    let mut next_id = 1u32;
    for row in &mut workbench.publish.history {
        if row.id == 0 {
            row.id = next_id;
        }
        next_id = next_id.max(row.id.saturating_add(1));
    }
    if workbench.publish.selected_history_id == Some(0) {
        workbench.publish.selected_history_id = None;
    }
}

fn payload_preview(payload: &[u8]) -> String {
    const LIMIT: usize = 96;
    let mut preview = String::from_utf8_lossy(payload).replace(['\n', '\r'], " ");
    if preview.len() > LIMIT {
        let truncated = preview.chars().take(LIMIT).collect::<String>();
        preview = format!("{truncated}...");
    }
    preview
}

fn mqtt_label(version: MqttVersion) -> &'static str {
    match version {
        MqttVersion::Mqtt311 => "MQTT 3.1.1",
        MqttVersion::Mqtt50 => "MQTT v5",
    }
}

fn qos(qos: StorageQos) -> QosLevel {
    match qos {
        StorageQos::AtMostOnce => QosLevel::Zero,
        StorageQos::AtLeastOnce => QosLevel::One,
        StorageQos::ExactlyOnce => QosLevel::Two,
    }
}

fn history_activity(history: &ConnectionHistorySnapshot) -> String {
    format!(
        "{} publish topic(s), {} subscription(s) migrated",
        history.publish_topics.topics.len(),
        history.subscriptions.topics.len()
    )
}

fn auth_label(connection: &ConnectionConfig) -> &'static str {
    match connection.auth {
        Auth::Off => "No Auth",
        Auth::Password => "Password",
        Auth::Keyfile => "Keyfile",
    }
}

fn password_status(connection: &ConnectionConfig, password: &SecretInput) -> &'static str {
    if !password.is_empty() {
        "MQTT password managed by keyring"
    } else if connection.username.is_some() {
        "MQTT password missing from keyring"
    } else {
        "No MQTT password configured"
    }
}

fn tls_label(connection: &ConnectionConfig) -> &'static str {
    match connection.ssl {
        TlsSsl::Off => "No TLS/SSL",
        TlsSsl::Keystore => "Keystore",
    }
}

fn tls_password_status(connection: &ConnectionConfig, password: &SecretInput) -> &'static str {
    if !password.is_empty() {
        "SSL password managed by keyring"
    } else if connection.ssl_keystore.is_some() {
        "SSL password missing from keyring"
    } else {
        "No SSL password configured"
    }
}

fn proxy_label(connection: &ConnectionConfig) -> &'static str {
    match connection.proxy {
        Proxy::Off => "No proxy/tunnel",
        Proxy::Ssh => "SSH",
    }
}

fn ssh_password_status(connection: &ConnectionConfig, password: &SecretInput) -> &'static str {
    if !password.is_empty() {
        return match connection.auth {
            Auth::Keyfile => "SSH key passphrase managed by keyring",
            _ => "SSH password managed by keyring",
        };
    }

    match connection.auth {
        Auth::Password => "SSH password missing from keyring",
        Auth::Keyfile => "No SSH key passphrase configured",
        Auth::Off => "No SSH password configured",
    }
}

