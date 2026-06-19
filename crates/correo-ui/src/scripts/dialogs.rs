use correo_core::{AppCommand, AppCommandSender, ScriptSurfaceSnapshot};
use egui::{Id, Modal, TextEdit, Ui};

use crate::{
    i18n::I18n, modal_style, theme::ThemeTokens, theme::CONTROL_HEIGHT, widgets::padded_text_edit,
};

pub(super) fn create_dialog(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if !scripts.create_dialog_open {
        return;
    }
    let response = modal_style::style(Modal::new(Id::new("create-script-modal")), tokens).show(
        ui.ctx(),
        |ui| {
            ui.set_width(360.0);
            ui.heading(i18n.text("script-new"));
            let mut name = scripts.new_script_name.clone();
            if ui
                .add_sized(
                    [ui.available_width(), CONTROL_HEIGHT],
                    padded_text_edit(TextEdit::singleline(&mut name)),
                )
                .changed()
            {
                send(commands, AppCommand::UpdateNewScriptName(name));
            }
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), CONTROL_HEIGHT),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if ui.button(i18n.text("common-save")).clicked() {
                        send(commands, AppCommand::CreateScript);
                    }
                    if ui.button(i18n.text("common-cancel")).clicked() {
                        send(commands, AppCommand::CancelCreateScript);
                    }
                },
            );
        },
    );
    if response.should_close() {
        send(commands, AppCommand::CancelCreateScript);
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
