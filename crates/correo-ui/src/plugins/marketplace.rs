use correo_core::{AppCommand, AppCommandSender, PluginMarketplaceRow, PluginSurfaceSnapshot};
use egui::{Button, Label, RichText, ScrollArea, Ui};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::responsive;
use crate::theme::ThemeTokens;
use correo_style::layout;

use crate::widgets::{
    fill_remaining_tile_rows, tile_list_content_width, tile_scroll_bar_rect_with_height,
};

use super::{
    install_button, marketplace_capability_chips, metadata_row, plugin_metadata_line, plugin_split,
    plugin_tile, search_field, send, TILE_HEIGHT,
};

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let filtered = plugins.filtered_marketplace_plugins();
    plugin_split(
        ui,
        plugins,
        tokens,
        commands,
        i18n,
        |ui| {
            marketplace_list(ui, plugins, &filtered, tokens, commands, i18n);
        },
        |ui| selected_detail(ui, plugins, tokens, commands, i18n),
    );
}

fn marketplace_list(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    filtered: &[&PluginMarketplaceRow],
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.heading(i18n.text("plugin-tab-marketplace"));
    ui.add_space(4.0);
    search_field(ui, plugins, commands, i18n);
    ui.add_space(8.0);
    let list_height = ui.available_height().max(layout::TABLE_MIN_HEIGHT);
    ScrollArea::vertical()
        .id_salt("plugin-marketplace-list")
        .max_height(list_height)
        .auto_shrink([false, false])
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, list_height))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(tile_list_content_width(ui));
            for (index, plugin) in filtered.into_iter().enumerate() {
                marketplace_row(ui, index, plugins, plugin, tokens, commands, i18n);
            }
            fill_remaining_tile_rows(ui, filtered.len(), TILE_HEIGHT, list_height, tokens);
        });
}

fn marketplace_row(
    ui: &mut Ui,
    index: usize,
    plugins: &PluginSurfaceSnapshot,
    plugin: &PluginMarketplaceRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let response = plugin_tile(
        ui,
        ("marketplace", plugin.id.as_str()),
        index,
        plugins.selected_marketplace_plugin_id == plugin.id,
        tokens,
        |ui| {
            ui.label(RichText::new(&plugin.name).strong());
            ui.add(
                Label::new(RichText::new(&plugin.description).color(tokens.text_secondary))
                    .truncate(),
            );
            let mut metadata = vec![plugin.version.clone(), plugin.provider.clone()];
            if plugin.installed_plugin_id.is_some() {
                metadata.push(i18n.text("plugin-installed"));
            }
            plugin_metadata_line(ui, &metadata, tokens);
            if !plugin.capabilities.is_empty() {
                marketplace_capability_chips(ui, &plugin.capabilities, tokens);
            }
        },
    );
    if response.clicked() {
        send(
            commands,
            AppCommand::SelectMarketplacePlugin(plugin.id.clone()),
        );
        responsive::close_plugin_flyout(ui.ctx());
    }
    response.context_menu(|ui| marketplace_context_menu(ui, plugins, plugin, commands, i18n));
}

fn marketplace_context_menu(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    plugin: &PluginMarketplaceRow,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if let Some(installed) = plugins.installed_plugin_for_marketplace(plugin) {
        if ui
            .add_enabled(
                installed.can_uninstall(),
                Button::new(menu_label(regular::TRASH, &i18n.text("plugin-uninstall"))),
            )
            .clicked()
        {
            send(
                commands,
                AppCommand::SelectMarketplacePlugin(plugin.id.clone()),
            );
            send(commands, AppCommand::SelectPlugin(installed.id.clone()));
            send(
                commands,
                AppCommand::UninstallPlugin {
                    plugin_id: installed.id.clone(),
                },
            );
            ui.close_menu();
        }
    } else if ui
        .button(menu_label(
            regular::DOWNLOAD_SIMPLE,
            &i18n.text("plugin-install"),
        ))
        .clicked()
    {
        send(
            commands,
            AppCommand::SelectMarketplacePlugin(plugin.id.clone()),
        );
        send(
            commands,
            AppCommand::InstallMarketplacePlugin {
                marketplace_plugin_id: plugin.id.clone(),
            },
        );
        ui.close_menu();
    }
}

fn menu_label(icon: &str, label: &str) -> String {
    format!("{icon}  {label}")
}

fn selected_detail(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let Some(plugin) = plugins.selected_marketplace_plugin() else {
        ui.heading(i18n.text("plugin-marketplace-detail"));
        ui.label(
            RichText::new(i18n.text("plugin-no-marketplace-selected")).color(tokens.text_secondary),
        );
        return;
    };
    ui.heading(&plugin.name);
    ui.label(format!(
        "{} {} {}",
        plugin.version,
        i18n.text("plugin-by"),
        plugin.provider
    ));
    ui.label(RichText::new(&plugin.repository).color(tokens.text_secondary));
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
        &plugin.location,
        tokens,
        i18n,
    );
    ui.add_space(8.0);
    action_bar(ui, plugin, commands, i18n);
    ui.add_space(8.0);
    marketplace_capability_chips(ui, &plugin.capabilities, tokens);
    if let Some(installed) = plugins.installed_plugin_for_marketplace(plugin) {
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "{} {}",
                i18n.text("plugin-installed-as"),
                installed.name
            ))
            .color(tokens.text_secondary),
        );
    }
}

fn action_bar(
    ui: &mut Ui,
    plugin: &PluginMarketplaceRow,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal_wrapped(|ui| {
        if plugin.installed_plugin_id.is_some() {
            ui.add_enabled(false, Button::new(i18n.text("plugin-installed")));
        } else {
            install_button(ui, &plugin.id, commands, i18n);
        }
    });
}
