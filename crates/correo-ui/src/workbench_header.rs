use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectDisabledReason, ConnectionState,
    ConnectionSummary,
};
use egui::{Button, RichText, Ui};
use egui_phosphor::regular;

use crate::{
    i18n::I18n,
    responsive,
    theme::ThemeTokens,
    widgets::{
        menu_item, menu_item_content_width, menu_item_enabled, set_menu_item_width,
        square_icon_button_size, with_icon_button_padding,
    },
};

pub fn connection_header(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let Some(connection) = snapshot.selected_connection() else {
        return;
    };
    if responsive::connections_context_is_compact(ui.ctx(), snapshot.active_workspace) {
        compact_connection_header(ui, snapshot, connection, tokens, commands, i18n);
        return;
    }

    ui.horizontal(|ui| {
        ui.heading(&connection.name);
        if header_icon_button(ui, regular::PENCIL_SIMPLE, "Edit connection").clicked() {
            send(commands, AppCommand::OpenConnectionSettings(connection.id));
        }
        if snapshot.plugins.has_connection_workflow_plugins()
            && header_icon_button(ui, regular::PUZZLE_PIECE, &i18n.text("validators-title"))
                .clicked()
        {
            send(commands, AppCommand::OpenConnectionPlugins(connection.id));
        }
        if header_icon_button(ui, regular::TRASH, "Delete connection").clicked() {
            send(commands, AppCommand::RequestDeleteConnection);
        }
        plugin_connection_actions(ui, snapshot, connection, commands);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            connection_action(ui, connection, commands, false);
            connection_summary(ui, connection, tokens);
        });
    });
}

fn compact_connection_header(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), square_icon_button_size()[1]),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let icon_actions = responsive::workbench_uses_icon_actions(ui.available_width());
            let connection_action_width = if icon_actions {
                square_icon_button_size()[0]
            } else {
                104.0
            };
            let center_width = (ui.available_width()
                - square_icon_button_size()[0]
                - connection_action_width
                - square_icon_button_size()[0]
                - (ui.spacing().item_spacing.x * 3.0))
                .max(80.0);
            if header_icon_button(ui, regular::LIST, "Show connections").clicked() {
                responsive::open_connection_flyout(ui.ctx());
            }
            connection_title(ui, connection, tokens, center_width);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                compact_overflow_menu(ui, snapshot, connection, commands, i18n);
                connection_action(ui, connection, commands, icon_actions);
            });
        },
    );
}

fn connection_title(ui: &mut Ui, connection: &ConnectionSummary, tokens: ThemeTokens, width: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, square_icon_button_size()[1]),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_clip_rect(ui.max_rect());
            ui.label(
                RichText::new(state_icon(connection.state))
                    .size(16.0)
                    .color(state_color(connection.state, tokens)),
            )
            .on_hover_text(connection.state.label());
            ui.label(RichText::new(&connection.name).strong().size(18.0))
                .on_hover_text(format!(
                    "{} · {}",
                    connection.state.label(),
                    connection.endpoint
                ));
        },
    );
}

fn header_icon_button(
    ui: &mut Ui,
    icon: &'static str,
    hover_text: impl Into<String>,
) -> egui::Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(icon).size(16.0)),
        )
    })
    .on_hover_text(hover_text.into())
}

fn connection_action(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    commands: &AppCommandSender,
    compact: bool,
) {
    let action = action_for(connection);
    let response = with_icon_button_padding(ui, |ui| {
        if compact {
            ui.add_enabled(
                action.enabled,
                Button::new(RichText::new(regular::PLUG).size(16.0)).min_size(egui::vec2(
                    square_icon_button_size()[0],
                    square_icon_button_size()[1],
                )),
            )
        } else {
            ui.add_enabled(
                action.enabled,
                Button::new(format!("{}  {}", regular::PLUG, action.label))
                    .min_size(egui::vec2(104.0, square_icon_button_size()[1])),
            )
        }
    })
    .on_hover_text(if compact {
        format!("{}: {}", action.label, action.tooltip)
    } else {
        action.tooltip
    });
    if response.clicked() {
        send(commands, action.command);
    }
}

fn compact_overflow_menu(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    connection: &ConnectionSummary,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let mut label_storage = vec![
        i18n.text("connection-edit-tooltip"),
        i18n.text("validators-title"),
        i18n.text("connection-delete-title"),
    ];
    label_storage.extend(
        snapshot
            .plugins
            .connection_header_actions()
            .into_iter()
            .map(|action| action.label.clone()),
    );
    let labels: Vec<&str> = label_storage.iter().map(String::as_str).collect();
    let menu_width = menu_item_content_width(ui, &labels);
    let response = with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(regular::DOTS_THREE_VERTICAL).size(16.0)),
        )
    });
    let response = response.on_hover_text("Connection actions");
    right_aligned_overflow_menu(ui, response, menu_width, |ui| {
        set_menu_item_width(ui, &labels);
        if menu_item(
            ui,
            Some(regular::PENCIL_SIMPLE),
            &i18n.text("connection-edit-tooltip"),
        )
        .clicked()
        {
            send(commands, AppCommand::OpenConnectionSettings(connection.id));
            return true;
        }
        if snapshot.plugins.has_connection_workflow_plugins()
            && menu_item(
                ui,
                Some(regular::PUZZLE_PIECE),
                &i18n.text("validators-title"),
            )
            .clicked()
        {
            send(commands, AppCommand::OpenConnectionPlugins(connection.id));
            return true;
        }
        if menu_item(
            ui,
            Some(regular::TRASH),
            &i18n.text("connection-delete-title"),
        )
        .clicked()
        {
            send(commands, AppCommand::RequestDeleteConnection);
            return true;
        }
        for action in snapshot.plugins.connection_header_actions() {
            let enabled = plugin_action_enabled(action.requires_connected, connection.state);
            if menu_item_enabled(ui, enabled, None, &action.label)
                .on_hover_text(&action.tooltip)
                .clicked()
            {
                send(
                    commands,
                    AppCommand::InvokeConnectionPluginAction {
                        plugin_id: action.plugin_id.clone(),
                        action_id: action.action_id.clone(),
                        connection_id: connection.id,
                    },
                );
                return true;
            }
        }
        false
    });
}

fn right_aligned_overflow_menu(
    ui: &mut Ui,
    response: egui::Response,
    content_width: f32,
    add_contents: impl FnOnce(&mut Ui) -> bool,
) {
    let popup_id = response.id.with("right-aligned-menu");
    if response.clicked() {
        ui.ctx().data_mut(|data| {
            let open = data.get_temp::<bool>(popup_id).unwrap_or(false);
            data.insert_temp(popup_id, !open);
        });
    }
    let open = ui
        .ctx()
        .data_mut(|data| data.get_temp::<bool>(popup_id).unwrap_or(false));
    if !open {
        return;
    }

    let frame = egui::Frame::menu(ui.style());
    let menu_width = content_width + frame.total_margin().sum().x;
    let mut pos = egui::pos2(response.rect.right() - menu_width, response.rect.bottom());
    pos.y += ui.spacing().menu_spacing;
    if let Some(to_global) = ui.ctx().layer_transform_to_global(response.layer_id) {
        pos = to_global * pos;
    }
    let area_response = egui::Area::new(popup_id)
        .kind(egui::UiKind::Menu)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| frame.show(ui, add_contents).inner);

    let should_close = ui.ctx().input(|input| input.key_pressed(egui::Key::Escape))
        || (response.clicked_elsewhere() && area_response.response.clicked_elsewhere())
        || area_response.inner;
    if should_close {
        ui.ctx().data_mut(|data| data.insert_temp(popup_id, false));
    }
}

fn plugin_connection_actions(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    connection: &ConnectionSummary,
    commands: &AppCommandSender,
) {
    for action in snapshot.plugins.connection_header_actions() {
        let enabled = plugin_action_enabled(action.requires_connected, connection.state);
        let response = with_icon_button_padding(ui, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                ui.add_sized(
                    [56.0, square_icon_button_size()[1]],
                    Button::new(RichText::new(&action.label).size(13.0)),
                )
            })
            .inner
        })
        .on_hover_text(&action.tooltip);
        if response.clicked() {
            send(
                commands,
                AppCommand::InvokeConnectionPluginAction {
                    plugin_id: action.plugin_id.clone(),
                    action_id: action.action_id.clone(),
                    connection_id: connection.id,
                },
            );
        }
    }
}

fn plugin_action_enabled(requires_connected: bool, state: ConnectionState) -> bool {
    !requires_connected || state == ConnectionState::Connected
}

fn connection_summary(ui: &mut Ui, connection: &ConnectionSummary, tokens: ThemeTokens) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(state_icon(connection.state))
                .size(22.0)
                .color(state_color(connection.state, tokens)),
        )
        .on_hover_text(connection.state.label());
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(
                    RichText::new(connection.state.label())
                        .color(state_color(connection.state, tokens)),
                )
                .on_hover_text(connection.state.label());
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

fn action_for(connection: &ConnectionSummary) -> HeaderAction {
    match connection.state {
        ConnectionState::Connected
        | ConnectionState::Connecting
        | ConnectionState::Reconnecting => HeaderAction::new(
            "Disconnect",
            "Disconnect from broker",
            true,
            AppCommand::Disconnect(connection.id),
        ),
        ConnectionState::Error => HeaderAction::new(
            "Reconnect",
            "Reconnect to broker",
            true,
            AppCommand::Reconnect(connection.id),
        ),
        ConnectionState::Disconnected => {
            let tooltip = if connection.can_connect() {
                "Connect to broker".to_owned()
            } else {
                disabled_reason(connection).label().to_owned()
            };
            HeaderAction::new(
                "Connect",
                tooltip,
                connection.can_connect(),
                AppCommand::Connect(connection.id),
            )
        }
    }
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
    label: &'static str,
    tooltip: String,
    enabled: bool,
    command: AppCommand,
}

impl HeaderAction {
    fn new(
        label: &'static str,
        tooltip: impl Into<String>,
        enabled: bool,
        command: AppCommand,
    ) -> Self {
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
