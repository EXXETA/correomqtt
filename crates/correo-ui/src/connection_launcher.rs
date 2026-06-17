use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionBadge, ConnectionState, ConnectionSummary,
};
use egui::{
    Button, Label, Layout, Rect, Response, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder,
};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::responsive;
use crate::theme::{ThemeTokens, CONTROL_HEIGHT};
use crate::widgets::{
    clearable_search_edit, disable_tile_text_selection, fill_remaining_tile_rows, menu_item,
    menu_item_enabled, set_menu_item_width, tighten_tile_spacing, tile_inner_padding,
    tile_list_content_width, tile_scroll_bar_rect_with_height, tile_table_fill,
    tile_table_hover_fill, with_icon_button_padding, TILE_GAP,
};
use correo_style::layout;

const STATUS_ICON_WIDTH: f32 = 26.0;
const STATUS_ICON_SIZE: f32 = 21.0;
const FEATURE_ICON_WIDTH: f32 = 19.0;
const FEATURE_ICON_SIZE: f32 = 17.0;
const FEATURE_ICON_GAP: f32 = 4.0;
const CONNECTION_ROW_TEXT_GAP: f32 = 8.0;
const CONNECTION_TEXT_GAP: f32 = CONNECTION_ROW_TEXT_GAP - 1.0;

pub fn panel(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(tile_list_content_width(ui), CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            if !responsive::forced_connection_flyout_mode(ui.ctx())
                && !responsive::connection_flyout_open(ui.ctx())
                && header_icon_button(ui, regular::LIST)
                    .on_hover_text("Use connections flyout")
                    .clicked()
            {
                responsive::set_forced_connection_flyout_mode(ui.ctx(), true);
                responsive::open_connection_flyout(ui.ctx());
            }
            ui.heading(i18n.text("connections-heading"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if header_add_button(ui)
                    .on_hover_text(i18n.text("common-add-connection"))
                    .clicked()
                {
                    send(commands, AppCommand::AddConnection);
                }
                if header_icon_button(ui, regular::EXPORT)
                    .on_hover_text(i18n.text("connection-export-tooltip"))
                    .clicked()
                {
                    send(commands, AppCommand::ExportConnections);
                }
                if header_icon_button(ui, regular::DOWNLOAD_SIMPLE)
                    .on_hover_text(i18n.text("connection-import-tooltip"))
                    .clicked()
                {
                    send(commands, AppCommand::ImportConnections);
                }
            });
        },
    );

    let mut filter = snapshot.connection_filter.clone();
    let response = clearable_search_edit(
        ui,
        None,
        &mut filter,
        "Search Connections",
        tile_list_content_width(ui),
    );
    if response.changed() {
        send(commands, AppCommand::SearchConnections(filter));
    }

    ui.add_space(8.0);
    let connections = snapshot.filtered_connections();
    let connection_count = connections.len();
    let row_height = connection_row_height(ui);
    let list_height = (ui.available_height() - layout::TABLE_SCROLL_BOTTOM_GAP).max(120.0);
    ScrollArea::vertical()
        .id_salt("connection-list")
        .max_height(list_height)
        .auto_shrink([false, false])
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, list_height))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(tile_list_content_width(ui));
            for (index, connection) in connections.into_iter().enumerate() {
                connection_row(
                    ui, index, connection, snapshot, tokens, commands, i18n, row_height,
                );
            }
            fill_remaining_tile_rows(ui, connection_count, row_height, list_height, tokens);
        });
}

fn connection_row_height(ui: &Ui) -> f32 {
    let top_padding = tile_inner_padding().y;
    (ui.text_style_height(&egui::TextStyle::Body) * 2.0)
        + CONNECTION_ROW_TEXT_GAP
        + (top_padding * 2.0)
}

fn header_add_button(ui: &mut Ui) -> Response {
    header_icon_button(ui, regular::PLUS)
}

fn header_icon_button(ui: &mut Ui, icon: &'static str) -> Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            crate::widgets::square_icon_button_size(),
            Button::new(RichText::new(icon).size(15.0)),
        )
    })
}

fn connection_row(
    ui: &mut Ui,
    index: usize,
    connection: &ConnectionSummary,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    row_height: f32,
) {
    let selected = snapshot.selected_connection == Some(connection.id);
    let row_width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(row_width, row_height), Sense::click_and_drag());
    response.dnd_set_drag_payload(connection.id);
    let dragged = response.dragged();
    let drop_target =
        response.contains_pointer() && egui::DragAndDrop::has_any_payload(ui.ctx()) && !dragged;

    let fill = if selected {
        tokens.accent_selected_bg
    } else if dragged {
        tokens.panel_raised
    } else if response.hovered() || response.contains_pointer() {
        tile_table_hover_fill(tokens)
    } else {
        tile_table_fill(index, tokens)
    };
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, fill);
    if dragged {
        ui.painter().rect_stroke(
            rect.shrink(2.0),
            egui::CornerRadius::ZERO,
            Stroke::new(2.0, tokens.accent),
            egui::StrokeKind::Inside,
        );
    }
    if drop_target {
        let after = ui
            .ctx()
            .pointer_interact_pos()
            .is_some_and(|pointer| pointer.y > rect.center().y);
        let y = if after {
            rect.bottom() - 2.0
        } else {
            rect.top() + 2.0
        };
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 6.0, y),
                egui::pos2(rect.right() - 6.0, y),
            ],
            Stroke::new(3.0, tokens.accent),
        );
    }

    let content_rect = rect.shrink2(tile_inner_padding());
    let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
    disable_tile_text_selection(&mut content_ui);
    tighten_tile_spacing(&mut content_ui);
    status_icon_at(
        &mut content_ui,
        connection.state,
        tokens,
        content_rect,
        &connection.name,
    )
    .on_hover_text(i18n.connection_state_label(connection.state));
    let info_rect = Rect::from_min_max(
        egui::pos2(
            content_rect.left() + STATUS_ICON_WIDTH + 8.0,
            content_rect.top(),
        ),
        content_rect.right_bottom(),
    );
    let mut info_ui = content_ui.new_child(UiBuilder::new().max_rect(info_rect));
    row_contents(&mut info_ui, connection, tokens, row_height);

    if let Some(dropped) = response.dnd_release_payload() {
        let connection_id = *dropped;
        if connection_id != connection.id {
            let after = response
                .interact_pointer_pos()
                .or_else(|| ui.ctx().pointer_interact_pos())
                .is_some_and(|pointer| pointer.y > response.rect.center().y);
            send(
                commands,
                AppCommand::MoveConnection {
                    connection_id,
                    target_connection_id: connection.id,
                    after,
                },
            );
        }
    }

    response.context_menu(|ui| connection_context_menu(ui, connection, snapshot, commands, i18n));

    if response.double_clicked() {
        if connection.can_connect() {
            send(commands, AppCommand::Connect(connection.id));
            close_compact_flyout(ui, snapshot);
        }
    } else if response.clicked() {
        send(commands, AppCommand::SelectConnection(connection.id));
        close_compact_flyout(ui, snapshot);
    }

    ui.add_space(TILE_GAP);
}

fn close_compact_flyout(ui: &Ui, snapshot: &AppSnapshot) {
    if responsive::connections_context_is_compact(ui.ctx(), snapshot.active_workspace) {
        responsive::close_connection_flyout(ui.ctx());
    }
}

fn connection_context_menu(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    snapshot: &AppSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let connect = i18n.text("common-connect");
    let disconnect = i18n.text("common-disconnect");
    let edit = i18n.text("common-edit");
    let validators = i18n.text("validators-title");
    let delete = i18n.text("common-delete");
    set_menu_item_width(ui, &[&disconnect, &connect, &edit, &validators, &delete]);
    match connection.state {
        ConnectionState::Connected => {
            if menu_item(ui, Some(regular::PLUG_CHARGING), &disconnect).clicked() {
                send(commands, AppCommand::SelectConnection(connection.id));
                send(commands, AppCommand::Disconnect(connection.id));
                ui.close_menu();
            }
        }
        _ => {
            if menu_item_enabled(ui, connection.can_connect(), Some(regular::PLUG), &connect)
                .clicked()
            {
                send(commands, AppCommand::SelectConnection(connection.id));
                send(commands, AppCommand::Connect(connection.id));
                ui.close_menu();
            }
        }
    }
    if menu_item(ui, Some(regular::PENCIL_SIMPLE), &edit).clicked() {
        send(commands, AppCommand::EditConnection(connection.id));
        ui.close_menu();
    }
    if snapshot.plugins.has_connection_workflow_plugins()
        && menu_item(ui, Some(regular::PUZZLE_PIECE), &validators).clicked()
    {
        send(commands, AppCommand::SelectConnection(connection.id));
        send(commands, AppCommand::OpenConnectionPlugins(connection.id));
        ui.close_menu();
    }
    if menu_item(ui, Some(regular::TRASH), &delete).clicked() {
        send(commands, AppCommand::SelectConnection(connection.id));
        send(commands, AppCommand::RequestDeleteConnection);
        ui.close_menu();
    }
}

fn row_contents(ui: &mut Ui, connection: &ConnectionSummary, tokens: ThemeTokens, row_height: f32) {
    let content_height = row_height - (tile_inner_padding().y * 2.0);
    ui.set_height(content_height);
    ui.allocate_ui(
        egui::vec2(ui.available_width().max(96.0), content_height),
        |ui| {
            connection_info(ui, connection, tokens);
        },
    );
}

fn status_icon_at(
    ui: &mut Ui,
    state: ConnectionState,
    tokens: ThemeTokens,
    content_rect: Rect,
    id_source: &str,
) -> Response {
    let rect = Rect::from_min_size(
        content_rect.left_top(),
        egui::vec2(STATUS_ICON_WIDTH, STATUS_ICON_SIZE + 2.0),
    );
    if ui.is_rect_visible(rect) {
        let galley = ui.painter().layout_no_wrap(
            state_icon(state).to_owned(),
            egui::FontId::proportional(STATUS_ICON_SIZE),
            state_color(state, tokens),
        );
        let pos = egui::pos2(
            rect.left() + (STATUS_ICON_WIDTH - galley.size().x) * 0.5,
            rect.top(),
        );
        ui.painter().galley(pos, galley, state_color(state, tokens));
    }
    ui.interact(
        rect,
        ui.id().with(("connection-status", id_source)),
        Sense::hover(),
    )
}

fn connection_info(ui: &mut Ui, connection: &ConnectionSummary, tokens: ThemeTokens) {
    let rect = ui.max_rect();
    let features = feature_icons(connection);
    let feature_width = feature_group_width(features.len());
    let right_width = feature_width;
    let right = rect.right();
    let right_left = (right - right_width).max(rect.left());
    let text_right = (right_left - 6.0).max(rect.left());
    let line_height = ui.text_style_height(&egui::TextStyle::Body);
    let title_rect = Rect::from_min_max(
        rect.left_top(),
        egui::pos2(text_right, rect.top() + line_height),
    );
    let endpoint_rect = Rect::from_min_max(
        egui::pos2(rect.left(), title_rect.bottom() + CONNECTION_TEXT_GAP),
        egui::pos2(
            text_right,
            title_rect.bottom() + CONNECTION_TEXT_GAP + line_height,
        ),
    );
    let feature_rect = Rect::from_min_max(
        egui::pos2((right - feature_width).max(right_left), rect.top()),
        egui::pos2(right, title_rect.bottom()),
    );
    let text_rect = Rect::from_min_max(rect.left_top(), endpoint_rect.right_bottom());
    connection_text_rows(ui, connection, tokens, text_rect);
    connection_feature_row(ui, features, tokens, feature_rect, line_height);
}

fn connection_text_rows(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    rect: Rect,
) {
    let mut text_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::top_down(egui::Align::Min)),
    );
    text_ui.set_width(rect.width());
    text_ui.spacing_mut().item_spacing.y = CONNECTION_TEXT_GAP;
    text_ui
        .add(Label::new(RichText::new(&connection.name).strong()).truncate())
        .on_hover_text(&connection.name);
    let endpoint = endpoint_label(connection);
    text_ui
        .add(Label::new(RichText::new(&endpoint).color(tokens.text_secondary)).truncate())
        .on_hover_text(&connection.endpoint);
}

fn connection_feature_row(
    ui: &mut Ui,
    features: Vec<FeatureIcon>,
    tokens: ThemeTokens,
    rect: Rect,
    line_height: f32,
) {
    let mut feature_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::right_to_left(egui::Align::Center)),
    );
    feature_ui.spacing_mut().item_spacing.x = FEATURE_ICON_GAP;
    for feature in features.into_iter().rev() {
        feature_icon(&mut feature_ui, feature, tokens, line_height);
    }
}

fn feature_icon(ui: &mut Ui, feature: FeatureIcon, tokens: ThemeTokens, line_height: f32) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(FEATURE_ICON_WIDTH, line_height), Sense::hover());
    if ui.is_rect_visible(rect) {
        let galley = ui.painter().layout_no_wrap(
            feature.icon.to_owned(),
            egui::FontId::proportional(FEATURE_ICON_SIZE),
            tokens.text_secondary,
        );
        let pos = egui::pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5,
        );
        ui.painter().galley(pos, galley, tokens.text_secondary);
    }
    response.on_hover_text(feature.label);
}

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

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
