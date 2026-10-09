use correo_style::layout;
use egui::{Context, Id, Ui};

use correo_core::Workspace;

pub(crate) fn workspace_has_sidebar(workspace: Workspace) -> bool {
    matches!(
        workspace,
        Workspace::Connections | Workspace::ImportExport | Workspace::Scripts | Workspace::Plugins
    )
}

pub(crate) fn sidebar_visible(ctx: &Context) -> bool {
    ctx.data_mut(|data| data.get_persisted(sidebar_visible_id()).unwrap_or(true))
}

pub(crate) fn toggle_sidebar(ctx: &Context) {
    let visible = sidebar_visible(ctx);
    ctx.data_mut(|data| data.insert_persisted(sidebar_visible_id(), !visible));
}

pub(crate) fn toggle_sidebar_shortcut(ctx: &Context, workspace: Workspace) -> bool {
    workspace_has_sidebar(workspace)
        && ctx.input(|input| {
            input.modifiers.ctrl && input.modifiers.shift && input.key_pressed(egui::Key::B)
        })
}

pub(crate) fn sidebar_uses_exclusive_content(ctx: &Context) -> bool {
    ctx.screen_rect().width() < layout::NARROW_SIDEBAR_WINDOW_WIDTH
}

pub(crate) fn set_workbench_tabs_visible(ctx: &Context, visible: bool) {
    ctx.data_mut(|data| data.insert_temp(workbench_tabs_id(), visible));
}

pub(crate) fn workbench_tabs_visible(ui: &Ui) -> bool {
    ui.ctx()
        .data_mut(|data| data.get_temp(workbench_tabs_id()).unwrap_or(false))
}

pub(crate) fn workbench_uses_icon_actions(width: f32) -> bool {
    width < layout::WORKBENCH_ICON_ONLY_ACTION_WIDTH
}

fn sidebar_visible_id() -> Id {
    Id::new("sidebar-visible")
}

fn workbench_tabs_id() -> Id {
    Id::new("workbench-narrow-tabs-visible")
}
