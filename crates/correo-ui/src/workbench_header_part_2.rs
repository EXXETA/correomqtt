fn plugin_action_enabled(requires_connected: bool, state: ConnectionState) -> bool {
    !requires_connected || state == ConnectionState::Connected
}

fn connection_summary(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(state_icon(connection.state))
                .size(22.0)
                .color(state_color(connection.state, tokens)),
        )
        .on_hover_text(i18n.connection_state_label(connection.state));
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(
                    RichText::new(i18n.connection_state_label(connection.state))
                        .color(state_color(connection.state, tokens)),
                )
                .on_hover_text(i18n.connection_state_label(connection.state));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(
                    RichText::new(&connection.endpoint)
                        .size(12.0)
                        .color(tokens.text_disabled),
                )
                .on_hover_text(&connection.endpoint);
            });
        });
    });
}

fn action_for(connection: &ConnectionSummary, i18n: &I18n) -> HeaderAction {
    match connection.state {
        ConnectionState::Connected
        | ConnectionState::Connecting
        | ConnectionState::Reconnecting => HeaderAction::new(
            i18n.text("common-disconnect"),
            i18n.text("connection-disconnect-tooltip"),
            true,
            AppCommand::Disconnect(connection.id),
        ),
        ConnectionState::Error => HeaderAction::new(
            i18n.text("common-reconnect"),
            i18n.text("connection-reconnect-tooltip"),
            true,
            AppCommand::Reconnect(connection.id),
        ),
        ConnectionState::Disconnected => {
            let tooltip = if connection.can_connect() {
                i18n.text("connection-connect-tooltip")
            } else {
                disabled_reason_label(disabled_reason(connection), i18n)
            };
            HeaderAction::new(
                i18n.text("common-connect"),
                tooltip,
                connection.can_connect(),
                AppCommand::Connect(connection.id),
            )
        }
    }
}

fn disabled_reason_label(reason: ConnectDisabledReason, i18n: &I18n) -> String {
    i18n.text(match reason {
        ConnectDisabledReason::AlreadyConnected => "disabled-already-connected",
        ConnectDisabledReason::MissingHost => "disabled-missing-host",
        ConnectDisabledReason::BrokerStopped => "disabled-broker-stopped",
        ConnectDisabledReason::Busy => "disabled-busy",
    })
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

fn disabled_reason(connection: &ConnectionSummary) -> ConnectDisabledReason {
    connection
        .disabled_reason
        .unwrap_or(ConnectDisabledReason::Busy)
}

struct HeaderAction {
    label: String,
    tooltip: String,
    enabled: bool,
    command: AppCommand,
}

impl HeaderAction {
    fn new(label: String, tooltip: impl Into<String>, enabled: bool, command: AppCommand) -> Self {
        Self {
            label,
            tooltip: tooltip.into(),
            enabled,
            command,
        }
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
