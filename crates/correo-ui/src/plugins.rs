use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, PluginDisableConfirmation, PluginLoadState,
    PluginRow, PluginStatus, PluginSurfaceSnapshot, PluginSurfaceTab,
};
use egui::{Button, RichText, Sense, Ui, UiBuilder, Window};

use crate::i18n::I18n;
use crate::widgets::{
    clearable_search_edit, disable_tile_text_selection, tighten_tile_spacing, tile_inner_padding,
    tile_list_content_width, tile_table_fill, tile_table_hover_fill, tile_table_selected_fill,
    TILE_GAP,
};
use crate::{motion, theme::ThemeTokens};
use std::hash::Hash;

#[path = "plugins/installed.rs"]
mod installed;
#[path = "plugins/keyboard.rs"]
mod keyboard;
#[path = "plugins/marketplace.rs"]
mod marketplace;

pub(super) const TILE_HEIGHT: f32 = 76.0;
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
            installed::detail(ui, &snapshot.plugins, tokens, commands, i18n)
        }
        PluginSurfaceTab::Marketplace => {
            marketplace::detail(ui, &snapshot.plugins, tokens, commands, i18n)
        }
        PluginSurfaceTab::Configuration
        | PluginSurfaceTab::Hooks
        | PluginSurfaceTab::Diagnostics => {
            installed::detail(ui, &snapshot.plugins, tokens, commands, i18n)
        }
    }
    if let Some(confirmation) = &snapshot.plugins.disable_confirmation {
        disable_confirmation(ui, confirmation, tokens, commands, i18n);
    }
}

pub fn sidebar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if snapshot.plugins.load_state != PluginLoadState::Ready {
        empty_state(ui, &snapshot.plugins, tokens, i18n);
        return;
    }
    ui.heading(i18n.text("plugin-header"));
    ui.add_space(8.0);
    toolbar(ui, &snapshot.plugins, tokens, commands, i18n);
    ui.add_space(8.0);
    match snapshot.plugins.active_tab {
        PluginSurfaceTab::Marketplace => {
            marketplace::sidebar(ui, &snapshot.plugins, tokens, commands, i18n)
        }
        _ => installed::sidebar(ui, &snapshot.plugins, tokens, commands, i18n),
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
