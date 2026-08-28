use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, PluginDisableConfirmation, PluginLoadState,
    PluginRow, PluginStatus, PluginSurfaceSnapshot, PluginSurfaceTab,
};
use egui::{Button, RichText, Sense, Stroke, Ui, UiBuilder, Window};
use egui_extras::{Size, StripBuilder};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::widgets::{
    clearable_search_edit, disable_tile_text_selection, tighten_tile_spacing, tile_inner_padding,
    tile_list_content_width, tile_table_fill, tile_table_hover_fill, tile_table_selected_fill,
    FLYOUT_HANDLE_WIDTH, TILE_GAP,
};
use crate::{modal_style, motion, responsive, theme::ThemeTokens};
use correo_style::layout;
use std::hash::Hash;

#[path = "plugins/installed.rs"]
mod installed;
#[path = "plugins/keyboard.rs"]
mod keyboard;
#[path = "plugins/marketplace.rs"]
mod marketplace;

pub(super) const TILE_HEIGHT: f32 = 76.0;
const LIST_WIDTH: f32 = layout::PLUGIN_FLYOUT_WIDTH;
const MIN_LIST_WIDTH: f32 = layout::PLUGIN_FLYOUT_WIDTH;
const MAX_LIST_WIDTH: f32 = 600.0;
const DETAIL_MIN_WIDTH: f32 = 550.0;
const SPLIT_GUTTER: f32 = 24.0;

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    keyboard::handle(ui.ctx(), &snapshot.plugins, commands);

    if snapshot.plugins.load_state != PluginLoadState::Ready {
        empty_state(ui, &snapshot.plugins, tokens, i18n);
        return;
    }
    match snapshot.plugins.active_tab {
        PluginSurfaceTab::Installed => {
            installed::tab(ui, &snapshot.plugins, tokens, commands, i18n)
        }
        PluginSurfaceTab::Marketplace => {
            marketplace::tab(ui, &snapshot.plugins, tokens, commands, i18n)
        }
        PluginSurfaceTab::Configuration
        | PluginSurfaceTab::Hooks
        | PluginSurfaceTab::Diagnostics => {
            installed::tab(ui, &snapshot.plugins, tokens, commands, i18n)
        }
    }
    if let Some(confirmation) = &snapshot.plugins.disable_confirmation {
        disable_confirmation(ui, confirmation, tokens, commands, i18n);
    }
}

fn toolbar(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        for tab in PluginSurfaceTab::ALL {
            if ui
                .selectable_label(plugins.active_tab == tab, i18n.plugin_tab_label(tab))
                .clicked()
            {
                send(commands, AppCommand::SelectPluginSurfaceTab(tab));
            }
        }
    });
    ui.label(RichText::new(plugin_counts(plugins, i18n)).color(tokens.text_secondary));
}

pub(super) fn plugin_metadata_line(ui: &mut Ui, parts: &[String], tokens: ThemeTokens) {
    ui.label(RichText::new(parts.join("  ")).color(tokens.text_secondary));
}

fn empty_state(ui: &mut Ui, plugins: &PluginSurfaceSnapshot, tokens: ThemeTokens, i18n: &I18n) {
    ui.add_space(24.0);
    ui.label(
        RichText::new(i18n.plugin_load_message(plugins.load_state)).color(tokens.text_secondary),
    );
}

fn disable_confirmation(
    ui: &mut Ui,
    confirmation: &PluginDisableConfirmation,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    Window::new(i18n.text("plugin-disable-title"))
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.label(RichText::new(&confirmation.plugin_name).strong());
            ui.label(i18n.text("plugin-disable-warning"));
            for hook in &confirmation.active_hooks {
                ui.label(RichText::new(hook.label()).color(tokens.warning));
            }
            ui.horizontal(|ui| {
                if ui.button(i18n.text("plugin-disable-action")).clicked() {
                    send(commands, AppCommand::ConfirmPluginDisable);
                }
                if ui.button(i18n.text("common-cancel")).clicked() {
                    send(commands, AppCommand::CancelPluginDisable);
                }
            });
        });
}

pub(super) fn plugin_detail(
    ui: &mut Ui,
    plugin: &PluginRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.heading(&plugin.name);
    ui.label(format!(
        "{} {} {}",
        plugin.version,
        i18n.text("plugin-by"),
        plugin.provider
    ));
    ui.label(RichText::new(i18n.plugin_source_label(plugin.source)).color(tokens.text_secondary));
    ui.label(
        RichText::new(i18n.plugin_status_label(plugin.status))
            .color(status_color(plugin.status, tokens)),
    );
    ui.add_space(8.0);
    ui.label(&plugin.description);
    metadata_row(
        ui,
        &i18n.text("plugin-license"),
        &plugin.license,
        tokens,
        i18n,
    );
    metadata_row(
        ui,
        &i18n.text("plugin-origin"),
        &plugin.origin,
        tokens,
        i18n,
    );
    metadata_row(
        ui,
        &i18n.text("plugin-path"),
        &plugin.installed_path,
        tokens,
        i18n,
    );
    if let Some(note) = &plugin.legacy_note {
        ui.label(RichText::new(note).color(tokens.warning));
    }
    ui.add_space(8.0);
    plugin_action_bar(ui, plugin, commands, i18n);
    ui.add_space(8.0);
    capability_chips(ui, plugin, tokens);
    plugin_operational_summary(ui, plugin, tokens, i18n);
}

pub(super) fn capability_chips(ui: &mut Ui, plugin: &PluginRow, tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        for capability in &plugin.capabilities {
            let color = if capability.granted {
                tokens.success
            } else {
                tokens.warning
            };
            ui.label(RichText::new(&capability.label).color(color));
        }
    });
}

pub(super) fn marketplace_capability_chips(
    ui: &mut Ui,
    capabilities: &[correo_core::PluginCapabilityRow],
    tokens: ThemeTokens,
) {
    ui.horizontal_wrapped(|ui| {
        for capability in capabilities {
            let color = if capability.granted {
                tokens.success
            } else {
                tokens.warning
            };
            ui.label(RichText::new(&capability.label).color(color));
        }
    });
}

pub(super) fn plugin_action_bar(
    ui: &mut Ui,
    plugin: &PluginRow,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal_wrapped(|ui| {
        let toggle_label = if plugin.enabled {
            i18n.text("plugin-disable")
        } else {
            i18n.text("plugin-enable")
        };
        if ui.button(toggle_label).clicked() {
            send(
                commands,
                AppCommand::SetPluginEnabled {
                    plugin_id: plugin.id.clone(),
                    enabled: !plugin.enabled,
                },
            );
        }
        if plugin.can_uninstall() && ui.add(Button::new(i18n.text("plugin-uninstall"))).clicked() {
            send(
                commands,
                AppCommand::UninstallPlugin {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
    });
}

pub(super) fn install_button(
    ui: &mut Ui,
    marketplace_plugin_id: &str,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if ui.button(i18n.text("plugin-install")).clicked() {
        send(
            commands,
            AppCommand::InstallMarketplacePlugin {
                marketplace_plugin_id: marketplace_plugin_id.to_owned(),
            },
        );
    }
}

pub(super) fn search_field(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let mut filter = plugins.plugin_filter.clone();
    if clearable_search_edit(
        ui,
        Some(keyboard::plugin_search_id()),
        &mut filter,
        i18n.text("plugin-search"),
        tile_list_content_width(ui),
    )
    .changed()
    {
        send(commands, AppCommand::SearchPlugins(filter));
    }
}

pub(super) fn plugin_tile(
    ui: &mut Ui,
    id_source: impl Hash,
    index: usize,
    selected: bool,
    tokens: ThemeTokens,
    add_contents: impl FnOnce(&mut Ui),
) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, TILE_HEIGHT), Sense::click());
    let fill = motion::tile_fill(
        ui,
        id_source,
        tile_table_fill(index, tokens),
        tile_table_hover_fill(tokens),
        tile_table_selected_fill(tokens),
        response.hovered() || response.contains_pointer(),
        selected,
    );
    let clip_rect = rect.intersect(ui.clip_rect());
    let painter = ui.painter().with_clip_rect(clip_rect);
    painter.rect_filled(rect, egui::CornerRadius::ZERO, fill);

    let content_rect = rect.shrink2(tile_inner_padding());
    let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
    disable_tile_text_selection(&mut content_ui);
    tighten_tile_spacing(&mut content_ui);
    content_ui.set_clip_rect(content_rect.intersect(clip_rect));
    add_contents(&mut content_ui);
    ui.add_space(TILE_GAP);
    response
}

pub(super) fn plugin_split(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    add_list: impl FnOnce(&mut Ui),
    add_detail: impl FnOnce(&mut Ui),
) {
    if responsive::plugin_context_is_compact(ui.ctx()) {
        add_detail(ui);
        plugin_flyout(ui.ctx(), plugins, tokens, commands, i18n, add_list);
        return;
    }

    let list_width = plugin_list_width(ui.available_width());
    StripBuilder::new(ui)
        .clip(true)
        .size(Size::exact(list_width))
        .size(Size::exact(SPLIT_GUTTER))
        .size(Size::remainder().at_least(DETAIL_MIN_WIDTH))
        .horizontal(|mut strip| {
            strip.cell(|ui| plugin_sidebar(ui, plugins, tokens, commands, i18n, add_list));
            strip.cell(|ui| divider(ui, tokens));
            strip.cell(add_detail);
        });
}

fn plugin_flyout(
    ctx: &egui::Context,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    add_list: impl FnOnce(&mut Ui),
) {
    let screen = ctx.screen_rect();
    let overlay_rect = egui::Rect::from_min_max(
        egui::pos2(
            screen.left() + layout::RAIL_WIDTH,
            screen.top() + layout::HEADER_HEIGHT,
        ),
        screen.right_bottom(),
    );
    if overlay_rect.width() <= 0.0 || overlay_rect.height() <= 0.0 {
        return;
    }

    let open = responsive::plugin_flyout_open(ctx);
    if !open {
        plugin_flyout_collapsed_handle(ctx, overlay_rect);
    }
    let Some(progress) = motion::flyout_progress(ctx, "plugin-context", open) else {
        return;
    };
    if open && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        responsive::close_plugin_flyout(ctx);
    }

    egui::Area::new(egui::Id::new("plugin-context-flyout"))
        .order(egui::Order::Foreground)
        .fixed_pos(overlay_rect.min)
        .movable(false)
        .show(ctx, |ui| {
            let (scrim_rect, _) = ui.allocate_exact_size(overlay_rect.size(), Sense::hover());
            ui.painter().rect_filled(
                scrim_rect,
                egui::CornerRadius::ZERO,
                motion::scrim_color(modal_style::SCRIM_ALPHA, progress),
            );

            let handle_rect = egui::Rect::from_min_size(
                scrim_rect.left_top(),
                egui::vec2(FLYOUT_HANDLE_WIDTH, scrim_rect.height()),
            );
            let panel_area = egui::Rect::from_min_max(
                egui::pos2(handle_rect.right(), scrim_rect.top()),
                scrim_rect.right_bottom(),
            );
            let panel_width = layout::PLUGIN_FLYOUT_WIDTH.min(panel_area.width());
            let panel_rect = motion::flyout_panel_rect(panel_area, panel_width, progress);
            ui.painter()
                .rect_filled(panel_rect, egui::CornerRadius::ZERO, tokens.window_bg);

            let margin = layout::sidebar_margin();
            let content_rect = egui::Rect::from_min_max(
                egui::pos2(
                    panel_rect.left() + f32::from(margin.left),
                    panel_rect.top() + f32::from(margin.top),
                ),
                egui::pos2(
                    panel_rect.right() - f32::from(margin.right),
                    panel_rect.bottom() - f32::from(margin.bottom),
                ),
            );
            let mut panel_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(content_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            panel_ui.multiply_opacity(motion::content_opacity(progress));
            panel_ui.set_clip_rect(content_rect);
            plugin_sidebar(&mut panel_ui, plugins, tokens, commands, i18n, add_list);
            plugin_flyout_expanded_controls(ui, handle_rect, panel_rect);

            let clicked_outside = open
                && ui.ctx().input(|input| {
                    input.pointer.any_click()
                        && input
                            .pointer
                            .interact_pos()
                            .is_some_and(|pos| !panel_rect.contains(pos))
                });
            if clicked_outside {
                responsive::close_plugin_flyout(ui.ctx());
            }
        });
}

fn plugin_flyout_collapsed_handle(ctx: &egui::Context, overlay_rect: egui::Rect) {
    egui::Area::new(egui::Id::new("plugin-context-flyout-collapsed-handle"))
        .order(egui::Order::Foreground)
        .fixed_pos(overlay_rect.min)
        .movable(false)
        .show(ctx, |ui| {
            let handle_rect = egui::Rect::from_min_size(
                ui.min_rect().min,
                egui::vec2(FLYOUT_HANDLE_WIDTH, overlay_rect.height()),
            );
            if crate::widgets::flyout_handle(
                ui,
                handle_rect,
                "plugin-flyout-open-handle",
                regular::CARET_RIGHT,
                "Show plugin list",
            )
            .clicked()
            {
                responsive::open_plugin_flyout(ui.ctx());
            }
        });
}

fn plugin_sidebar(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    add_list: impl FnOnce(&mut Ui),
) {
    ui.horizontal(|ui| {
        ui.heading(i18n.text("plugin-header"));
    });
    ui.add_space(8.0);
    toolbar(ui, plugins, tokens, commands, i18n);
    ui.add_space(8.0);
    add_list(ui);
}

fn plugin_flyout_expanded_controls(ui: &mut Ui, handle_rect: egui::Rect, panel_rect: egui::Rect) {
    if crate::widgets::flyout_handle(
        ui,
        handle_rect,
        "plugin-flyout-collapse-handle",
        regular::CARET_LEFT,
        "Collapse plugin list",
    )
    .clicked()
    {
        responsive::close_plugin_flyout(ui.ctx());
    }

    if !responsive::forced_plugin_flyout_mode(ui.ctx())
        || responsive::plugin_context_requires_flyout(ui.ctx())
    {
        return;
    }

    if crate::widgets::flyout_restore_button_above_edge(
        ui,
        "plugin-flyout-restore-button",
        panel_rect.right(),
        panel_rect.top(),
        "Use plugin sidebar",
    )
    .clicked()
    {
        responsive::set_forced_context_flyout_mode(ui.ctx(), false);
        responsive::close_plugin_flyout(ui.ctx());
    }
}

fn plugin_list_width(available_width: f32) -> f32 {
    let max_for_detail = (available_width - SPLIT_GUTTER - DETAIL_MIN_WIDTH).max(MIN_LIST_WIDTH);
    LIST_WIDTH.clamp(MIN_LIST_WIDTH, MAX_LIST_WIDTH.min(max_for_detail))
}

fn divider(ui: &mut Ui, tokens: ThemeTokens) {
    let rect = ui.max_rect();
    let x = rect.center().x;
    ui.painter().line_segment(
        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
        Stroke::new(1.0, tokens.border),
    );
    if crate::widgets::flyout_mode_button_above_divider(
        ui,
        "plugin-flyout-mode-button",
        rect,
        "Use plugin flyout",
    )
    .clicked()
    {
        responsive::set_forced_context_flyout_mode(ui.ctx(), true);
        responsive::close_plugin_flyout(ui.ctx());
        motion::finish_flyout_closed(ui.ctx(), "plugin-context");
    }
}

pub(super) fn metadata_row(
    ui: &mut Ui,
    label: &str,
    value: &str,
    tokens: ThemeTokens,
    i18n: &I18n,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(format!("{label}:")).strong());
        ui.label(RichText::new(metadata_value(value, i18n)).color(tokens.text_secondary));
    });
}

fn plugin_operational_summary(ui: &mut Ui, plugin: &PluginRow, tokens: ThemeTokens, i18n: &I18n) {
    ui.add_space(8.0);
    ui.label(
        RichText::new(format!(
            "{} {}",
            plugin.hooks.len(),
            i18n.text("plugin-hook-assignments")
        ))
        .color(tokens.text_secondary),
    );
    if !plugin.config_fields.is_empty() {
        ui.label(
            RichText::new(format!(
                "{} {}",
                plugin.config_fields.len(),
                i18n.text("plugin-config-fields")
            ))
            .color(tokens.text_secondary),
        );
    }
}

fn plugin_counts(plugins: &PluginSurfaceSnapshot, i18n: &I18n) -> String {
    let enabled = plugins
        .plugins
        .iter()
        .filter(|plugin| plugin.enabled)
        .count();
    format!(
        "{} {}, {enabled} {}",
        plugins.plugins.len(),
        i18n.text("plugin-installed-word"),
        i18n.text("plugin-enabled-word")
    )
}

pub(super) fn status_color(status: PluginStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        PluginStatus::Active => tokens.success,
        PluginStatus::Disabled => tokens.text_secondary,
        PluginStatus::NeedsConfig
        | PluginStatus::CapabilityDenied
        | PluginStatus::UnsupportedLegacy => tokens.warning,
        PluginStatus::LoadError | PluginStatus::HookFailed => tokens.danger,
    }
}

fn metadata_value(value: &str, i18n: &I18n) -> String {
    let value = value.trim();
    if value.is_empty() {
        i18n.text("plugin-not-recorded")
    } else {
        value.to_owned()
    }
}

pub(super) fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
