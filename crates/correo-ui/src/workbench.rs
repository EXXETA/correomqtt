use correo_core::{AppCommandSender, AppSnapshot};
use egui::Ui;

use crate::{
    i18n::I18n, theme::ThemeTokens, workbench_dialogs, workbench_header, workbench_layout,
    workbench_messages, workbench_plugin_windows, workbench_publish, workbench_subscribe,
    PayloadHighlighter,
};

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    payload_highlighter: Option<&PayloadHighlighter>,
) {
    if snapshot.selected_connection().is_none() {
        ui.label(i18n.text("connection-none-available"));
        return;
    }

    workbench_header::connection_header(ui, snapshot, tokens, commands, i18n);
    ui.add_space(6.0);
    workbench_layout::show(
        ui,
        tokens,
        snapshot.workbench.narrow_tab,
        |ui| workbench_publish::editor(ui, snapshot, tokens, commands, payload_highlighter),
        |ui| workbench_subscribe::editor(ui, snapshot, tokens, commands),
        |ui| workbench_publish::outgoing_messages(ui, snapshot, tokens, commands, i18n),
        |ui| workbench_subscribe::incoming_messages(ui, snapshot, tokens, commands, i18n),
    );
    workbench_messages::show(
        ui.ctx(),
        snapshot,
        tokens,
        commands,
        i18n,
        payload_highlighter,
    );
    workbench_plugin_windows::show(ui.ctx(), snapshot, tokens, commands);
    workbench_dialogs::unsubscribe_all_confirmation(ui, snapshot, commands);
}
