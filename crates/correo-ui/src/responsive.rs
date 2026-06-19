use correo_core::Workspace;
use correo_style::layout;
use egui::{Context, Id, Ui};

pub(crate) fn connections_context_is_compact(ctx: &Context, active_workspace: Workspace) -> bool {
    active_workspace == Workspace::Connections
        && (forced_connection_flyout_mode(ctx) || connections_context_requires_flyout(ctx))
}

pub(crate) fn connections_context_requires_flyout(ctx: &Context) -> bool {
    ctx.screen_rect().width() < layout::CONNECTION_CONTEXT_FLYOUT_SCREEN_WIDTH
}

pub(crate) fn forced_connection_flyout_mode(ctx: &Context) -> bool {
    ctx.data_mut(|data| data.get_persisted(forced_flyout_id()).unwrap_or(false))
}

pub(crate) fn set_forced_context_flyout_mode(ctx: &Context, enabled: bool) {
    ctx.data_mut(|data| {
        data.insert_persisted(forced_flyout_id(), enabled);
        data.insert_persisted(forced_scripting_flyout_id(), enabled);
        data.insert_persisted(forced_plugin_flyout_id(), enabled);
    });
}

pub(crate) fn scripting_context_is_compact(ctx: &Context) -> bool {
    forced_scripting_flyout_mode(ctx) || scripting_context_requires_flyout(ctx)
}

pub(crate) fn scripting_context_requires_flyout(ctx: &Context) -> bool {
    ctx.screen_rect().width() < layout::SCRIPTING_CONTEXT_FLYOUT_SCREEN_WIDTH
}

pub(crate) fn forced_scripting_flyout_mode(ctx: &Context) -> bool {
    ctx.data_mut(|data| {
        data.get_persisted(forced_scripting_flyout_id())
            .unwrap_or(false)
    })
}

pub(crate) fn plugin_context_is_compact(ctx: &Context) -> bool {
    forced_plugin_flyout_mode(ctx) || plugin_context_requires_flyout(ctx)
}

pub(crate) fn plugin_context_requires_flyout(ctx: &Context) -> bool {
    ctx.screen_rect().width() < layout::PLUGIN_CONTEXT_FLYOUT_SCREEN_WIDTH
}

pub(crate) fn forced_plugin_flyout_mode(ctx: &Context) -> bool {
    ctx.data_mut(|data| {
        data.get_persisted(forced_plugin_flyout_id())
            .unwrap_or(false)
    })
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

pub(crate) fn connection_flyout_open(ctx: &Context) -> bool {
    ctx.data_mut(|data| data.get_temp(flyout_id()).unwrap_or(false))
}

pub(crate) fn open_connection_flyout(ctx: &Context) {
    ctx.data_mut(|data| data.insert_temp(flyout_id(), true));
}

pub(crate) fn close_connection_flyout(ctx: &Context) {
    ctx.data_mut(|data| data.insert_temp(flyout_id(), false));
}

pub(crate) fn scripting_flyout_open(ctx: &Context) -> bool {
    ctx.data_mut(|data| data.get_temp(scripting_flyout_id()).unwrap_or(false))
}

pub(crate) fn open_scripting_flyout(ctx: &Context) {
    ctx.data_mut(|data| data.insert_temp(scripting_flyout_id(), true));
}

pub(crate) fn close_scripting_flyout(ctx: &Context) {
    ctx.data_mut(|data| data.insert_temp(scripting_flyout_id(), false));
}

pub(crate) fn plugin_flyout_open(ctx: &Context) -> bool {
    ctx.data_mut(|data| data.get_temp(plugin_flyout_id()).unwrap_or(false))
}

pub(crate) fn open_plugin_flyout(ctx: &Context) {
    ctx.data_mut(|data| data.insert_temp(plugin_flyout_id(), true));
}

pub(crate) fn close_plugin_flyout(ctx: &Context) {
    ctx.data_mut(|data| data.insert_temp(plugin_flyout_id(), false));
}

fn flyout_id() -> Id {
    Id::new("connections-context-flyout-open")
}

fn forced_flyout_id() -> Id {
    Id::new("connections-context-force-flyout")
}

fn scripting_flyout_id() -> Id {
    Id::new("scripting-context-flyout-open")
}

fn forced_scripting_flyout_id() -> Id {
    Id::new("scripting-context-force-flyout")
}

fn plugin_flyout_id() -> Id {
    Id::new("plugin-context-flyout-open")
}

fn forced_plugin_flyout_id() -> Id {
    Id::new("plugin-context-force-flyout")
}

fn workbench_tabs_id() -> Id {
    Id::new("workbench-narrow-tabs-visible")
}
