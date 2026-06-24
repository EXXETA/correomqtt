use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectDisabledReason, ConnectionState,
    ConnectionSummary,
};
use correo_style::layout;
use egui::{Button, Layout, Rect, RichText, Sense, Ui, UiBuilder};
use egui_phosphor::regular;

use crate::{
    i18n::I18n,
    responsive,
    theme::ThemeTokens,
    widgets::{
        menu_item, menu_item_content_width, menu_item_enabled, menu_item_with_activity_dot,
        paint_icon_activity_dot, set_menu_item_width, square_icon_button_size,
        with_icon_button_padding,
    },
    workbench_layout,
};

const TABBED_HEADER_COMPACT_ACTIONS_WIDTH: f32 = 960.0;

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
    if responsive::connections_context_is_compact(ui.ctx(), snapshot.active_workspace)
        || tabbed_header_needs_compact_actions(ui)
    {
        compact_connection_header(ui, snapshot, connection, tokens, commands, i18n);
        return;
    }

    let rect = Rect::from_min_size(
        ui.available_rect_before_wrap().min,
        egui::vec2(ui.available_width(), square_icon_button_size()[1]),
    );
    ui.allocate_rect(rect, Sense::hover());

    let mut left = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(egui::Align::Center)),
    );
    left.set_clip_rect(rect);
    left.heading(&connection.name);
    if !connection.immutable {
        if header_icon_button(
            &mut left,
            regular::PENCIL_SIMPLE,
            i18n.text("connection-edit-tooltip"),
        )
        .clicked()
        {
            send(commands, AppCommand::OpenConnectionSettings(connection.id));
        }
    }
    if snapshot.plugins.has_connection_workflow_plugins()
        && header_icon_button_with_activity_dot(
            &mut left,
            regular::PUZZLE_PIECE,
            &i18n.text("validators-title"),
            connection.active_plugin_workflows,
            tokens.accent,
        )
        .clicked()
    {
        send(commands, AppCommand::OpenConnectionPlugins(connection.id));
    }
    if !connection.immutable {
        if header_icon_button(
            &mut left,
            regular::TRASH,
            i18n.text("connection-delete-title"),
        )
        .clicked()
        {
            send(commands, AppCommand::RequestDeleteConnection);
        }
    }
    plugin_connection_actions(&mut left, snapshot, connection, commands);

    title_tab_controls_if_visible(ui, rect, snapshot, tokens, commands);

    let mut right = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::right_to_left(egui::Align::Center)),
    );
    right.set_clip_rect(rect);
    connection_action(&mut right, connection, commands, i18n, false);
    connection_summary(&mut right, connection, tokens, i18n);
}

fn compact_connection_header(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let response = ui.allocate_ui_with_layout(
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
                - connection_action_width
                - square_icon_button_size()[0]
                - square_icon_button_size()[0]
                - (ui.spacing().item_spacing.x * 3.0))
                .max(80.0);
            connection_title(ui, connection, tokens, i18n, center_width, false);
            compact_overflow_menu(ui, snapshot, connection, tokens, commands, i18n);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                connection_action(ui, connection, commands, i18n, icon_actions);
                connection_state_icon(ui, connection, tokens, i18n, 22.0);
            });
        },
    );
    title_tab_controls_if_visible(ui, response.response.rect, snapshot, tokens, commands);
}

fn tabbed_header_needs_compact_actions(ui: &Ui) -> bool {
    let width = ui.available_width();
    workbench_layout::tabs_visible_for_width(ui, width)
        && width < TABBED_HEADER_COMPACT_ACTIONS_WIDTH
}

fn title_tab_controls_if_visible(
    ui: &mut Ui,
    rect: Rect,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if !workbench_layout::tabs_visible_for_width(ui, rect.width()) {
        return;
    }

    let width = workbench_layout::title_tab_controls_width(rect.width());
    let tab_rect = Rect::from_center_size(rect.center(), egui::vec2(width, layout::CONTROL_HEIGHT));
    workbench_layout::title_bar_tab_controls(
        ui,
        tab_rect,
        tokens,
        snapshot.workbench.narrow_tab,
        commands,
        workbench_layout::natural_tabs_for_width(rect.width()),
    );
}

fn connection_title(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    i18n: &I18n,
    width: f32,
    show_state_icon: bool,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, square_icon_button_size()[1]),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_clip_rect(ui.max_rect());
            if show_state_icon {
                connection_state_icon(ui, connection, tokens, i18n, 16.0);
            }
            ui.label(RichText::new(&connection.name).strong().size(18.0))
                .on_hover_text(format!(
                    "{} · {}",
                    i18n.connection_state_label(connection.state),
                    connection.endpoint
                ));
        },
    );
}

fn connection_state_icon(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    i18n: &I18n,
    size: f32,
) -> egui::Response {
    ui.label(
        RichText::new(state_icon(connection.state))
            .size(size)
            .color(state_color(connection.state, tokens)),
    )
    .on_hover_text(i18n.connection_state_label(connection.state))
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

fn header_icon_button_with_activity_dot(
    ui: &mut Ui,
    icon: &'static str,
    hover_text: impl Into<String>,
    active: bool,
    dot_color: egui::Color32,
) -> egui::Response {
    let response = header_icon_button(ui, icon, hover_text);
    if active {
        paint_icon_activity_dot(ui, response.rect.center(), 16.0, dot_color);
    }
    response
}

fn connection_action(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    commands: &AppCommandSender,
    i18n: &I18n,
    compact: bool,
) {
    let action = action_for(connection, i18n);
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
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let mut label_storage = vec![i18n.text("validators-title")];
    if !connection.immutable {
        label_storage.push(i18n.text("connection-edit-tooltip"));
        label_storage.push(i18n.text("connection-delete-title"));
    }
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
    let response = response.on_hover_text(i18n.text("connection-actions"));
    aligned_overflow_menu(ui, response, menu_width, MenuAlignment::Left, |ui| {
        set_menu_item_width(ui, &labels);
        if !connection.immutable {
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
        }
        if snapshot.plugins.has_connection_workflow_plugins()
            && connection_workflow_menu_item(
                ui,
                &i18n.text("validators-title"),
                connection.active_plugin_workflows,
                tokens.accent,
            )
            .clicked()
        {
            send(commands, AppCommand::OpenConnectionPlugins(connection.id));
            return true;
        }
        if !connection.immutable {
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

fn connection_workflow_menu_item(
    ui: &mut Ui,
    label: &str,
    active: bool,
    dot_color: egui::Color32,
) -> egui::Response {
    if active {
        menu_item_with_activity_dot(ui, Some(regular::PUZZLE_PIECE), label, dot_color)
    } else {
        menu_item(ui, Some(regular::PUZZLE_PIECE), label)
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum MenuAlignment {
    Left,
    Right,
}

fn aligned_overflow_menu(
    ui: &mut Ui,
    response: egui::Response,
    content_width: f32,
    alignment: MenuAlignment,
    add_contents: impl FnOnce(&mut Ui) -> bool,
) {
    let popup_id = response.id.with("aligned-menu");
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
    let x = match alignment {
        MenuAlignment::Left => response.rect.left(),
        MenuAlignment::Right => response.rect.right() - menu_width,
    };
    let mut pos = egui::pos2(x, response.rect.bottom());
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
