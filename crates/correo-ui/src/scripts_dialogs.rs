fn delete_dialog(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if !scripts.delete_confirmation_open {
        return;
    }
    let response = crate::modal_style::style(Modal::new(Id::new("delete-script-modal")), tokens)
        .show(ui.ctx(), |ui| {
            ui.set_width(360.0);
            ui.heading(i18n.text("script-delete"));
            ui.label(i18n.text_with_args(
                "script-delete-detail",
                &[("name", scripts.selected_script.clone())],
            ));
            ui.horizontal(|ui| {
                if ui.button(i18n.text("common-cancel")).clicked() {
                    send(commands, AppCommand::CancelDeleteScript);
                }
                if ui.button(i18n.text("common-delete")).clicked() {
                    send(commands, AppCommand::ConfirmDeleteScript);
                }
            });
        });
    if response.should_close() {
        send(commands, AppCommand::CancelDeleteScript);
    }
}

fn file_status_color(status: ScriptFileStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        ScriptFileStatus::Ready => tokens.success,
        ScriptFileStatus::Dirty => tokens.warning,
        ScriptFileStatus::Running => tokens.script,
        ScriptFileStatus::Error => tokens.danger,
    }
}

fn execution_color(status: ScriptExecutionStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        ScriptExecutionStatus::Queued | ScriptExecutionStatus::Running => tokens.script,
        ScriptExecutionStatus::Succeeded => tokens.success,
        ScriptExecutionStatus::Failed => tokens.danger,
        ScriptExecutionStatus::Cancelled => tokens.warning,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
