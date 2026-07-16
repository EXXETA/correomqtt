fn feature_icons(connection: &ConnectionSummary) -> Vec<FeatureIcon> {
    let mut icons = vec![mqtt_feature_icon(&connection.mqtt_version)];
    for badge in &connection.badges {
        if let Some(icon) = badge_feature_icon(*badge) {
            icons.push(icon);
        }
    }
    icons
}

fn badge_feature_icon(badge: ConnectionBadge) -> Option<FeatureIcon> {
    match badge {
        ConnectionBadge::Credentials => Some(FeatureIcon::new(regular::KEY, "Credentials set")),
        ConnectionBadge::Tls => Some(FeatureIcon::new(regular::LOCK_KEY, "TLS/SSL")),
        ConnectionBadge::Proxy => Some(FeatureIcon::new(regular::SUBWAY, "Tunnel")),
        ConnectionBadge::Lwt => None,
    }
}

fn mqtt_feature_icon(version: &str) -> FeatureIcon {
    if version.contains('5') {
        FeatureIcon::new(regular::NUMBER_CIRCLE_FIVE, "MQTT 5")
    } else {
        FeatureIcon::new(regular::NUMBER_CIRCLE_THREE, "MQTT 3")
    }
}

fn feature_group_width(count: usize) -> f32 {
    if count == 0 {
        0.0
    } else {
        (count as f32 * FEATURE_ICON_WIDTH) + ((count - 1) as f32 * FEATURE_ICON_GAP)
    }
}

fn endpoint_label(connection: &ConnectionSummary) -> String {
    if has_tunnel(connection) {
        format!("via {}", connection.endpoint)
    } else {
        connection.endpoint.clone()
    }
}

fn has_tunnel(connection: &ConnectionSummary) -> bool {
    connection.badges.contains(&ConnectionBadge::Proxy)
}

fn state_icon(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connected
        | ConnectionState::Connecting
        | ConnectionState::Reconnecting => regular::WIFI_HIGH,
        ConnectionState::Disconnected | ConnectionState::Error => regular::WIFI_SLASH,
    }
}

fn state_color(state: ConnectionState, tokens: ThemeTokens) -> egui::Color32 {
    match state {
        ConnectionState::Connected => tokens.success,
        ConnectionState::Connecting | ConnectionState::Reconnecting => tokens.warning,
        ConnectionState::Error => tokens.danger,
        ConnectionState::Disconnected => tokens.text_secondary,
    }
}

#[derive(Clone, Copy)]
struct FeatureIcon {
    icon: &'static str,
    label: &'static str,
}

impl FeatureIcon {
    fn new(icon: &'static str, label: &'static str) -> Self {
        Self { icon, label }
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mqtt_feature_uses_number_circle_icons() {
        assert_eq!(
            mqtt_feature_icon("MQTT v5").icon,
            regular::NUMBER_CIRCLE_FIVE
        );
        assert_eq!(
            mqtt_feature_icon("MQTT 3.1.1").icon,
            regular::NUMBER_CIRCLE_THREE
        );
    }

    #[test]
    fn badge_features_match_connection_tile_spec() {
        assert_eq!(
            badge_feature_icon(ConnectionBadge::Credentials)
                .expect("credentials icon")
                .icon,
            regular::KEY
        );
        assert_eq!(
            badge_feature_icon(ConnectionBadge::Tls)
                .expect("tls icon")
                .icon,
            regular::LOCK_KEY
        );
        assert_eq!(
            badge_feature_icon(ConnectionBadge::Proxy)
                .expect("tunnel icon")
                .icon,
            regular::SUBWAY
        );
        assert!(badge_feature_icon(ConnectionBadge::Lwt).is_none());
    }
}
