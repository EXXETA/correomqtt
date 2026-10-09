use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, BuiltInBrokerStatus, ConnectionState,
    ScriptExecutionStatus, Workspace,
};
use egui::{Align, Align2, Button, Color32, CornerRadius, FontId, Layout, Ui};

use crate::i18n::I18n;
use crate::icons;
use crate::responsive;
use crate::theme::ThemeTokens;
use crate::widgets::{dotted_focus_outline, square_icon_button_size, with_icon_button_padding};
use egui_phosphor::regular;

const TOP_WORKSPACES: [Workspace; 3] = [
    Workspace::Connections,
    Workspace::Scripts,
    Workspace::Broker,
];
const BOTTOM_WORKSPACES: [Workspace; 4] = [
    Workspace::Plugins,
    Workspace::Diagnostics,
    Workspace::Settings,
    Workspace::About,
];
const NAV_BUTTON_GAP: f32 = 4.0;
const BOTTOM_RAIL_PADDING: f32 = 24.0;
const STATUS_DOT_RADIUS: f32 = 4.0;

pub fn rail(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space(4.0);
        sidebar_toggle(ui, snapshot, i18n);
        ui.add_space(12.0);
        nav_group(ui, &TOP_WORKSPACES, snapshot, tokens, commands, i18n);
        let button_height = square_icon_button_size()[1];
        let bottom_height =
            BOTTOM_WORKSPACES.len() as f32 * (button_height + NAV_BUTTON_GAP) + BOTTOM_RAIL_PADDING;
        ui.add_space((ui.available_height() - bottom_height).max(0.0));
        nav_group(ui, &BOTTOM_WORKSPACES, snapshot, tokens, commands, i18n);
    });
}

fn sidebar_toggle(ui: &mut Ui, snapshot: &AppSnapshot, i18n: &I18n) {
    let available = responsive::workspace_has_sidebar(snapshot.active_workspace);
    let visible = responsive::sidebar_visible(ui.ctx());
    let tooltip = if available {
        i18n.text(if visible {
            "sidebar-hide"
        } else {
            "sidebar-show"
        })
    } else {
        i18n.text("sidebar-unavailable")
    };
    let response = with_icon_button_padding(ui, |ui| {
        ui.add_enabled_ui(available, |ui| {
            ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
            ui.add_sized(
                square_icon_button_size(),
                Button::new(regular::SIDEBAR_SIMPLE).corner_radius(CornerRadius::same(4)),
            )
        })
        .inner
    })
    .on_hover_text(tooltip);
    if response.has_focus() {
        dotted_focus_outline(ui, response.rect);
    }
    if response.clicked() {
        responsive::toggle_sidebar(ui.ctx());
    }
}

fn nav_group(
    ui: &mut Ui,
    workspaces: &[Workspace],
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    for &workspace in workspaces {
        nav_button(ui, workspace, snapshot, tokens, commands, i18n);
        ui.add_space(NAV_BUTTON_GAP);
    }
}

fn nav_button(
    ui: &mut Ui,
    workspace: Workspace,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let selected = workspace == snapshot.active_workspace;
    let response = with_icon_button_padding(ui, |ui| {
        if !selected {
            ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        }
        let mut button = Button::new("").corner_radius(CornerRadius::same(4));
        if selected {
            button = button.fill(tokens.accent_selected_bg);
        }
        ui.add_sized(square_icon_button_size(), button)
    })
    .on_hover_text(i18n.workspace_label(workspace));
    let visuals = ui.style().interact_selectable(&response, selected);
    ui.painter().text(
        response.rect.center(),
        Align2::CENTER_CENTER,
        icons::workspace_icon(workspace),
        FontId::proportional(18.0),
        visuals.fg_stroke.color,
    );
    if selected {
        let rect = response.rect;
        let accent = egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height()));
        ui.painter()
            .rect_filled(accent, CornerRadius::same(1), tokens.accent);
    }
    if let Some(color) = nav_status_color(workspace, snapshot, tokens) {
        paint_status_dot(ui, response.rect, color);
    }
    if response.has_focus() {
        dotted_focus_outline(ui, response.rect);
    }
    if response.clicked() {
        let _ = commands.send(AppCommand::SelectWorkspace(workspace));
    }
}

fn nav_status_color(
    workspace: Workspace,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
) -> Option<Color32> {
    match workspace {
        Workspace::Connections => connection_status_color(snapshot, tokens),
        Workspace::Scripts => snapshot
            .scripts
            .executions
            .iter()
            .any(|execution| execution.status == ScriptExecutionStatus::Running)
            .then_some(tokens.success),
        Workspace::Broker => (snapshot.built_in_broker.status == BuiltInBrokerStatus::Running)
            .then_some(tokens.success),
        _ => None,
    }
}

fn connection_status_color(snapshot: &AppSnapshot, tokens: ThemeTokens) -> Option<Color32> {
    let connected = snapshot.connections.iter().any(|connection| {
        matches!(
            connection.state,
            ConnectionState::Connected | ConnectionState::Reconnecting
        )
    });
    let failed = snapshot
        .connections
        .iter()
        .any(|connection| connection.state == ConnectionState::Error);
    match (connected, failed) {
        (true, true) => Some(tokens.warning),
        (true, false) => Some(tokens.success),
        (false, true) => Some(tokens.danger),
        (false, false) => None,
    }
}

fn paint_status_dot(ui: &Ui, rect: egui::Rect, color: Color32) {
    let center = egui::pos2(rect.right() - 7.0, rect.top() + 7.0);
    ui.painter()
        .circle_filled(center, STATUS_DOT_RADIUS + 1.5, ui.visuals().panel_fill);
    ui.painter().circle_filled(center, STATUS_DOT_RADIUS, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_groups_match_sidebar_spec() {
        assert_eq!(
            TOP_WORKSPACES,
            [
                Workspace::Connections,
                Workspace::Scripts,
                Workspace::Broker
            ]
        );
        assert_eq!(
            BOTTOM_WORKSPACES,
            [
                Workspace::Plugins,
                Workspace::Diagnostics,
                Workspace::Settings,
                Workspace::About,
            ]
        );
    }
}
