use correo_core::{
    AppCommand, AppCommandSender, ConnectionSecretField, ConnectionSettingField,
    ConnectionSettingFlag, SecretInput,
};
use correo_style::layout;
use egui::{Button, ComboBox, TextEdit, Ui};

use crate::theme::CONTROL_HEIGHT;

pub(super) const FORM_MAX_WIDTH: f32 = LABEL_WIDTH + layout::TOOLBAR_GAP + CONTROL_MAX_WIDTH;
const LABEL_WIDTH: f32 = 180.0;
const CONTROL_MAX_WIDTH: f32 = layout::SETTINGS_CONTROL_WIDTH;

pub(super) fn field(
    ui: &mut Ui,
    label: &str,
    value: &str,
    field: ConnectionSettingField,
    commands: &AppCommandSender,
) {
    row(ui, label, |ui| {
        let mut edited = value.to_owned();
        if ui
            .add_sized(
                [control_width(ui), CONTROL_HEIGHT],
                crate::widgets::padded_text_edit(TextEdit::singleline(&mut edited)),
            )
            .changed()
        {
            send(
                commands,
                AppCommand::UpdateConnectionSetting {
                    field,
                    value: edited,
                },
            );
        }
    });
}

pub(super) fn field_with_button(
    ui: &mut Ui,
    label: &str,
    value: &str,
    field: ConnectionSettingField,
    button: &str,
    command: AppCommand,
    commands: &AppCommandSender,
) {
    row(ui, label, |ui| {
        let mut edited = value.to_owned();
        let button_width = row_button_width(ui, button);
        let field_width = (control_width(ui) - button_width - ui.spacing().item_spacing.x).max(0.0);
        if ui
            .add_sized(
                [field_width, CONTROL_HEIGHT],
                crate::widgets::padded_text_edit(TextEdit::singleline(&mut edited)),
            )
            .changed()
        {
            send(
                commands,
                AppCommand::UpdateConnectionSetting {
                    field,
                    value: edited,
                },
            );
        }
        if ui
            .add_sized([button_width, CONTROL_HEIGHT], Button::new(button))
            .clicked()
        {
            send(commands, command);
        }
    });
}

pub(super) fn combo(
    ui: &mut Ui,
    label: &str,
    value: &str,
    field: ConnectionSettingField,
    options: &[&str],
    commands: &AppCommandSender,
) {
    row(ui, label, |ui| {
        let mut selected = value.to_owned();
        ComboBox::from_id_salt(label)
            .selected_text(value)
            .width(control_width(ui))
            .show_ui(ui, |ui| {
                for option in options {
                    ui.selectable_value(&mut selected, (*option).to_owned(), *option);
                }
            });
        if selected != value {
            send(
                commands,
                AppCommand::UpdateConnectionSetting {
                    field,
                    value: selected,
                },
            );
        }
    });
}

pub(super) fn flag(
    ui: &mut Ui,
    label: &str,
    value: bool,
    flag: ConnectionSettingFlag,
    commands: &AppCommandSender,
) {
    let mut enabled = value;
    row(ui, label, |ui| {
        if crate::widgets::checkbox(ui, &mut enabled, "").changed() {
            send(
                commands,
                AppCommand::SetConnectionSettingFlag { flag, enabled },
            );
        }
    });
}

pub(super) fn secret_field(
    ui: &mut Ui,
    label: &str,
    value: &SecretInput,
    field: ConnectionSecretField,
    commands: &AppCommandSender,
) {
    secret_field_enabled(ui, label, value, field, true, commands);
}

pub(super) fn secret_field_enabled(
    ui: &mut Ui,
    label: &str,
    value: &SecretInput,
    field: ConnectionSecretField,
    enabled: bool,
    commands: &AppCommandSender,
) {
    ui.add_enabled_ui(enabled, |ui| {
        row(ui, label, |ui| {
            let mut edited = value.expose_for_ui().to_owned();
            if ui
                .add_sized(
                    [control_width(ui), CONTROL_HEIGHT],
                    crate::widgets::padded_text_edit(TextEdit::singleline(&mut edited))
                        .password(true),
                )
                .changed()
            {
                send(
                    commands,
                    AppCommand::UpdateConnectionSecret {
                        field,
                        value: SecretInput::new(edited),
                    },
                );
            }
        });
    });
}

pub(super) fn file_field(
    ui: &mut Ui,
    label: &str,
    value: &str,
    field: ConnectionSettingField,
    enabled: bool,
    commands: &AppCommandSender,
) {
    ui.add_enabled_ui(enabled, |ui| {
        row(ui, label, |ui| {
            let mut edited = value.to_owned();
            let button_width = row_button_width(ui, "Choose...");
            let field_width =
                (control_width(ui) - button_width - ui.spacing().item_spacing.x).max(0.0);
            if ui
                .add_sized(
                    [field_width, CONTROL_HEIGHT],
                    crate::widgets::padded_text_edit(TextEdit::singleline(&mut edited)),
                )
                .changed()
            {
                send(
                    commands,
                    AppCommand::UpdateConnectionSetting {
                        field,
                        value: edited,
                    },
                );
            }
            if ui
                .add_sized([button_width, CONTROL_HEIGHT], Button::new("Choose..."))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    send(
                        commands,
                        AppCommand::UpdateConnectionSetting {
                            field,
                            value: path.display().to_string(),
                        },
                    );
                }
            }
        });
    });
}

pub(super) fn row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(CONTROL_HEIGHT);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(label_width(ui), CONTROL_HEIGHT),
            egui::Sense::hover(),
        );
        ui.painter().text(
            egui::pos2(rect.left(), rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::TextStyle::Body.resolve(ui.style()),
            ui.visuals().text_color(),
        );
        add(ui);
    });
}

pub(super) fn control_width(ui: &Ui) -> f32 {
    ui.available_width().clamp(0.0, CONTROL_MAX_WIDTH)
}

fn label_width(ui: &Ui) -> f32 {
    LABEL_WIDTH.min((ui.available_width() * 0.4).max(120.0))
}

fn row_button_width(ui: &Ui, label: &str) -> f32 {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, ui.visuals().text_color())
        .size()
        .x;
    (text_width + ui.spacing().button_padding.x * 2.0).max(96.0)
}

pub(super) fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
