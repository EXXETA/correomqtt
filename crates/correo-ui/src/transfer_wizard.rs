use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionExportSnapshot, ConnectionImportSnapshot,
    ConnectionSurface, SecretInput, TransferSection, TransferStep,
};
use egui::{Button, Id, Modal, RichText, TextEdit, Ui};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::modal_style;
use crate::theme::{ThemeTokens, CONTROL_HEIGHT};
use crate::transfer_wizard_rows::{connection_list, select_buttons};
use crate::widgets::{checkbox, padded_text_edit};

const DIALOG_WIDTH: f32 = 900.0;
const DIALOG_RATIO: f32 = 5.0 / 3.0;
const DIALOG_INNER_PADDING: i8 = 24;
const CONTENT_HEIGHT: f32 = 340.0;
const LIST_WIDTH: f32 = 260.0;

#[derive(Default)]
pub(crate) struct State {
    import_password: String,
    export_password: String,
}

pub(crate) fn show(
    context: &egui::Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    state: &mut State,
) {
    if snapshot.connection_surface != ConnectionSurface::Transfer {
        state.import_password.clear();
        state.export_password.clear();
        return;
    }

    let content_size = modal_content_size(context);
    let response = modal_style::style(Modal::new(Id::new("connection-transfer-modal")), tokens)
        .frame(
            egui::Frame::NONE
                .fill(modal_bg(tokens))
                .corner_radius(egui::CornerRadius::same(4))
                .inner_margin(egui::Margin::same(DIALOG_INNER_PADDING)),
        )
        .show(context, |ui| {
            ui.set_min_size(content_size);
            ui.set_max_size(content_size);
            match snapshot.transfer.active_section {
                TransferSection::Import | TransferSection::Messages => import_wizard(
                    ui,
                    &snapshot.transfer.import,
                    snapshot.transfer.active_step,
                    tokens,
                    commands,
                    i18n,
                    state,
                ),
                TransferSection::Export => {
                    export_wizard(ui, &snapshot.transfer.export, tokens, commands, i18n, state)
                }
            }
        });

    if response.should_close() {
        send(commands, AppCommand::OpenConnectionLauncher);
    }
}

fn import_wizard(
    ui: &mut Ui,
    import: &ConnectionImportSnapshot,
    active_step: TransferStep,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    state: &mut State,
) {
    wizard_title(ui, regular::SIGN_IN, &i18n.text("transfer-import-title"));
    ui.add_space(20.0);
    match active_step {
        TransferStep::ChooseFile => import_explanation(ui, commands, i18n),
        TransferStep::Password => import_password(ui, import, tokens, commands, i18n, state),
        TransferStep::Review => import_review(ui, import, tokens, commands, i18n),
        TransferStep::Complete => success_view(
            ui,
            &i18n.text("transfer-import-title"),
            &i18n.text("transfer-import-success-title"),
            &count_text(
                i18n,
                "transfer-import-success-detail",
                import.selected_count(),
            ),
            commands,
            i18n,
        ),
    }
}

fn import_explanation(ui: &mut Ui, commands: &AppCommandSender, i18n: &I18n) {
    ui.label(RichText::new(i18n.text("transfer-import-explain-title")).strong());
    ui.add_space(8.0);
    ui.label(i18n.text("transfer-import-explain-detail"));
    footer(ui, |ui| {
        cancel_button(ui, commands, i18n);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button(format!(
                    "{} {}",
                    regular::ARROW_RIGHT,
                    i18n.text("transfer-import-choose-file")
                ))
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("CorreoMQTT connection", &["cqc"])
                    .pick_file()
                {
                    send(commands, AppCommand::ChooseConnectionImportFile(path));
                }
            }
        });
    });
}

fn import_password(
    ui: &mut Ui,
    import: &ConnectionImportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    state: &mut State,
) {
    ui.label(RichText::new(i18n.text("transfer-import-decrypt-title")).strong());
    ui.add_space(8.0);
    ui.label(
        RichText::new(i18n.text("transfer-import-decrypt-detail")).color(tokens.text_secondary),
    );
    ui.add_space(24.0);
    ui.horizontal(|ui| {
        ui.label(i18n.text("connection-password"));
        ui.add_sized(
            [ui.available_width(), CONTROL_HEIGHT],
            padded_text_edit(TextEdit::singleline(&mut state.import_password)).password(true),
        );
    });
    if import.password_state == correo_core::ImportPasswordState::InvalidRecoverable {
        ui.label(RichText::new(import.password_state.label()).color(tokens.danger));
    }
    footer(ui, |ui| {
        cancel_button(ui, commands, i18n);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can_decrypt = !state.import_password.is_empty();
            if ui
                .add_enabled(
                    can_decrypt,
                    Button::new(format!(
                        "{} {}",
                        regular::LOCK_OPEN,
                        i18n.text("transfer-import-decrypt-action")
                    )),
                )
                .clicked()
            {
                let password = state.import_password.clone();
                state.import_password.clear();
                send(
                    commands,
                    AppCommand::SubmitConnectionImportPassword(password),
                );
            }
            if ui
                .button(format!(
                    "{} {}",
                    regular::ARROW_LEFT,
                    i18n.text("common-back")
                ))
                .clicked()
            {
                send(
                    commands,
                    AppCommand::SelectTransferStep(TransferStep::ChooseFile),
                );
            }
        });
    });
}

fn import_review(
    ui: &mut Ui,
    import: &ConnectionImportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    two_column(
        ui,
        |ui| {
            connection_list(ui, &import.rows, tokens, commands, i18n, true);
        },
        |ui| {
            ui.label(RichText::new(i18n.text("transfer-import-review-title")).strong());
            ui.add_space(8.0);
            ui.label(
                RichText::new(i18n.text("transfer-import-review-detail"))
                    .color(tokens.text_secondary),
            );
            ui.add_space(18.0);
            select_buttons(ui, &import.rows, commands, i18n, true);
        },
    );
    footer(ui, |ui| {
        cancel_button(ui, commands, i18n);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    import.selected_count() > 0,
                    Button::new(format!(
                        "{} {}",
                        regular::SIGN_IN,
                        i18n.text("transfer-import-action")
                    )),
                )
                .clicked()
            {
                send(commands, AppCommand::StartConnectionImport);
            }
            if ui
                .button(format!(
                    "{} {}",
                    regular::ARROW_LEFT,
                    i18n.text("common-back")
                ))
                .clicked()
            {
                send(
                    commands,
                    AppCommand::SelectTransferStep(TransferStep::ChooseFile),
                );
            }
        });
    });
}

fn export_wizard(
    ui: &mut Ui,
    export: &ConnectionExportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    state: &mut State,
) {
    wizard_title(ui, regular::SIGN_OUT, &i18n.text("transfer-export-title"));
    ui.add_space(20.0);
    if export
        .outcome
        .as_ref()
        .is_some_and(|outcome| outcome.success)
    {
        success_view(
            ui,
            &i18n.text("transfer-export-title"),
            "",
            &count_text(
                i18n,
                "transfer-export-success-detail",
                export.selected_count(),
            ),
            commands,
            i18n,
        );
        return;
    }

    two_column(
        ui,
        |ui| {
            connection_list(ui, &export.rows, tokens, commands, i18n, false);
        },
        |ui| {
            ui.label(RichText::new(i18n.text("transfer-export-review-title")).strong());
            ui.add_space(8.0);
            ui.label(
                RichText::new(i18n.text("transfer-export-review-detail"))
                    .color(tokens.text_secondary),
            );
            ui.add_space(18.0);
            select_buttons(ui, &export.rows, commands, i18n, false);
            ui.add_space(28.0);
            export_encryption(ui, export, tokens, commands, i18n, state);
        },
    );
    footer(ui, |ui| {
        cancel_button(ui, commands, i18n);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can_export = export.selected_count() > 0
                && (!export.encrypted || !state.export_password.is_empty());
            if ui
                .add_enabled(
                    can_export,
                    Button::new(format!(
                        "{} {}",
                        regular::EXPORT,
                        i18n.text("transfer-export-action")
                    )),
                )
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("CorreoMQTT connection", &["cqc"])
                    .set_file_name("connections.cqc")
                    .save_file()
                {
                    send(
                        commands,
                        AppCommand::UpdateConnectionExportPath(path.display().to_string()),
                    );
                    let password = SecretInput::new(state.export_password.clone());
                    state.export_password.clear();
                    send(commands, AppCommand::StartConnectionExport { password });
                }
            }
        });
    });
}

fn export_encryption(
    ui: &mut Ui,
    export: &ConnectionExportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    state: &mut State,
) {
    ui.label(RichText::new(i18n.text("transfer-export-encryption-title")).strong());
    ui.add_space(8.0);
    ui.label(
        RichText::new(i18n.text("transfer-export-encryption-detail")).color(tokens.text_secondary),
    );
    ui.add_space(24.0);
    ui.horizontal(|ui| {
        ui.label(i18n.text("transfer-export-use-encryption"));
        let mut encrypted = export.encrypted;
        if checkbox(ui, &mut encrypted, "").changed() {
            send(
                commands,
                AppCommand::SetConnectionExportEncrypted(encrypted),
            );
        }
    });
    if export.encrypted {
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.label(i18n.text("connection-password"));
            ui.add_sized(
                [ui.available_width(), CONTROL_HEIGHT],
                padded_text_edit(TextEdit::singleline(&mut state.export_password)).password(true),
            );
        });
    }
}

fn success_view(
    ui: &mut Ui,
    title: &str,
    heading: &str,
    detail: &str,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if !heading.is_empty() {
        ui.label(RichText::new(heading).strong());
        ui.add_space(8.0);
    }
    ui.label(detail);
    ui.add_space(CONTENT_HEIGHT - 80.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button(i18n.text("common-ok")).clicked() {
            send(commands, AppCommand::OpenConnectionLauncher);
        }
        ui.label(RichText::new(title).color(ui.visuals().weak_text_color()));
    });
}

fn two_column(ui: &mut Ui, left: impl FnOnce(&mut Ui), right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(LIST_WIDTH, CONTENT_HEIGHT),
            egui::Layout::top_down(egui::Align::Min),
            left,
        );
        ui.separator();
        ui.add_space(18.0);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), CONTENT_HEIGHT),
            egui::Layout::top_down(egui::Align::Min),
            right,
        );
    });
}

fn wizard_title(ui: &mut Ui, icon: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).size(32.0));
        ui.heading(title);
    });
}

fn footer(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    ui.add_space((ui.available_height() - CONTROL_HEIGHT).max(0.0));
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        add,
    );
}

fn cancel_button(ui: &mut Ui, commands: &AppCommandSender, i18n: &I18n) {
    if ui
        .button(format!(
            "{} {}",
            regular::PROHIBIT,
            i18n.text("common-cancel")
        ))
        .clicked()
    {
        send(commands, AppCommand::OpenConnectionLauncher);
    }
}

fn count_text(i18n: &I18n, key: &str, count: usize) -> String {
    i18n.text(key).replace("{count}", &count.to_string())
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}

fn modal_content_size(context: &egui::Context) -> egui::Vec2 {
    let max_outer = context.screen_rect().size() - egui::vec2(64.0, 64.0);
    let mut outer_width = DIALOG_WIDTH
        .min(max_outer.x)
        .min(max_outer.y * DIALOG_RATIO);
    let min_width = 640.0_f32.min(max_outer.x);
    outer_width = outer_width.max(min_width);
    let outer_height = outer_width / DIALOG_RATIO;
    let padding = f32::from(DIALOG_INNER_PADDING) * 2.0;
    egui::vec2(outer_width - padding, outer_height - padding)
}

fn modal_bg(tokens: ThemeTokens) -> egui::Color32 {
    let color = tokens.window_bg;
    egui::Color32::from_rgb(color.r(), color.g(), color.b())
}
