use correo_core::{AppCommand, AppCommandSender, ConnectionSettingsSnapshot};
use egui::{Align, Button, Id, Layout, Modal, Ui};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::modal_style;
use crate::theme::{ThemeTokens, CONTROL_HEIGHT};

use super::controls::FORM_MAX_WIDTH;

pub(super) fn action_bar(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
    modal: bool,
    allow_delete: bool,
) {
    let width = ui.available_width().min(FORM_MAX_WIDTH);
    ui.allocate_ui_with_layout(
        egui::vec2(width, CONTROL_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_width(width);
            if allow_delete
                && ui
                    .button(format!(
                        "{}  {}...",
                        regular::TRASH,
                        i18n.text("common-delete")
                    ))
                    .clicked()
            {
                send(commands, AppCommand::RequestDeleteConnection);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let cancel_label = if modal {
                    i18n.text("common-cancel")
                } else {
                    i18n.text("common-discard")
                };
                if ui
                    .add_enabled(
                        settings.dirty || modal,
                        Button::new(format!("{}  {cancel_label}", regular::X)),
                    )
                    .clicked()
                {
                    send(commands, AppCommand::DiscardConnectionSettings);
                }
                let can_save = settings.dirty;
                let save = ui.add_enabled(
                    can_save,
                    Button::new(format!(
                        "{}  {}",
                        regular::FLOPPY_DISK,
                        i18n.text("common-save")
                    )),
                );
                if save.clicked() {
                    send(commands, AppCommand::SaveConnectionSettings);
                }
            });
        },
    );
}

pub(super) fn delete_confirmation(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let response = modal_style::style(Modal::new(Id::new("delete-connection-modal")), tokens).show(
        ui.ctx(),
        |ui| {
            ui.set_width(360.0);
            ui.heading(i18n.text("connection-delete-title"));
            ui.label(format!(
                "{} {}?",
                i18n.text("common-delete"),
                settings.profile_name
            ));
            ui.label(i18n.text("connection-delete-detail"));
            ui.horizontal(|ui| {
                if ui.button(i18n.text("common-cancel")).clicked() {
                    send(commands, AppCommand::CancelDeleteConnection);
                }
                if ui.button(i18n.text("common-delete")).clicked() {
                    send(commands, AppCommand::ConfirmDeleteConnection);
                }
            });
        },
    );
    if response.should_close() {
        send(commands, AppCommand::CancelDeleteConnection);
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
