fn validate_optional_port(value: &str, label: &str, required: bool, errors: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        if required {
            errors.push(format!("{label} is required"));
        }
        return;
    }
    match trimmed.parse::<u16>() {
        Ok(0) | Err(_) => errors.push(format!("{label} must be between 1 and 65535")),
        Ok(_) => {}
    }
}

fn connection_summary(
    id: ConnectionId,
    settings: &ConnectionSettingsSnapshot,
) -> ConnectionSummary {
    ConnectionSummary {
        id,
        name: settings.profile_name.trim().to_owned(),
        endpoint: format!("{}:{}", settings.host.trim(), settings.port.trim()),
        mqtt_version: settings.mqtt_version.clone(),
        badges: connection_badges(settings),
        active_plugin_workflows: active_plugin_workflows(settings),
        state: ConnectionState::Disconnected,
        disabled_reason: settings
            .host
            .trim()
            .is_empty()
            .then_some(ConnectDisabledReason::MissingHost),
        immutable: false,
        recent_subscriptions: 0,
        recent_messages: 0,
        last_activity: "Ready".to_owned(),
    }
}

fn active_plugin_workflows(settings: &ConnectionSettingsSnapshot) -> bool {
    settings
        .plugin_workflows
        .iter()
        .any(|workflow| workflow.enabled)
}

fn connection_badges(settings: &ConnectionSettingsSnapshot) -> Vec<ConnectionBadge> {
    let mut badges = Vec::new();
    if !settings.username.trim().is_empty() || !settings.password.is_empty() {
        badges.push(ConnectionBadge::Credentials);
    }
    if settings.tls_mode != "No TLS/SSL" {
        badges.push(ConnectionBadge::Tls);
    }
    if settings.proxy_mode != "No proxy/tunnel" {
        badges.push(ConnectionBadge::Proxy);
    }
    if settings.lwt_enabled {
        badges.push(ConnectionBadge::Lwt);
    }
    badges
}
