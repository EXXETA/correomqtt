use correo_core::{AppCommand, AppCommandSender, Workspace};
use egui::{Align, Align2, Button, Color32, CornerRadius, FontId, Layout, Ui};

use crate::i18n::I18n;
use crate::icons;
use crate::motion;
use crate::responsive;
use crate::theme::ThemeTokens;
use crate::widgets::{dotted_focus_outline, square_icon_button_size, with_icon_button_padding};

const TOP_WORKSPACES: [Workspace; 2] = [Workspace::Connections, Workspace::Scripts];
const BOTTOM_WORKSPACES: [Workspace; 4] = [
    Workspace::Plugins,
    Workspace::Diagnostics,
    Workspace::Settings,
    Workspace::About,
];
const NAV_BUTTON_GAP: f32 = 4.0;
const BOTTOM_RAIL_PADDING: f32 = 24.0;
pub fn rail(
    ui: &mut Ui,
    active: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space(4.0);
        nav_group(ui, &TOP_WORKSPACES, active, tokens, commands, i18n);
        let button_height = square_icon_button_size()[1];
        let bottom_height =
            BOTTOM_WORKSPACES.len() as f32 * (button_height + NAV_BUTTON_GAP) + BOTTOM_RAIL_PADDING;
        ui.add_space((ui.available_height() - bottom_height).max(0.0));
        nav_group(ui, &BOTTOM_WORKSPACES, active, tokens, commands, i18n);
    });
}

fn nav_group(
    ui: &mut Ui,
    workspaces: &[Workspace],
    active: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    for &workspace in workspaces {
        nav_button(ui, workspace, active, tokens, commands, i18n);
        ui.add_space(NAV_BUTTON_GAP);
    }
}

fn nav_button(
    ui: &mut Ui,
    workspace: Workspace,
    active: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let selected = workspace == active;
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
    if response.has_focus() {
        dotted_focus_outline(ui, response.rect);
    }
    if response.clicked() {
        close_flyouts_without_animation(ui);
        let _ = commands.send(AppCommand::SelectWorkspace(workspace));
    }
}

fn close_flyouts_without_animation(ui: &Ui) {
    responsive::close_connection_flyout(ui.ctx());
    responsive::close_scripting_flyout(ui.ctx());
    responsive::close_plugin_flyout(ui.ctx());
    motion::finish_flyout_closed(ui.ctx(), "connections-context");
    motion::finish_flyout_closed(ui.ctx(), "scripting-context");
    motion::finish_flyout_closed(ui.ctx(), "plugin-context");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_groups_match_sidebar_spec() {
        assert_eq!(TOP_WORKSPACES, [Workspace::Connections, Workspace::Scripts]);
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
