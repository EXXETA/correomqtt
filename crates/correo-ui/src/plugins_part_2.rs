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
