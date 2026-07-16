fn multiline(
    ui: &mut Ui,
    index: usize,
    field: ConnectionPluginWorkflowField,
    value: &str,
    commands: &AppCommandSender,
) {
    let mut value = value.to_owned();
    if ui
        .add(
            padded_text_edit(TextEdit::multiline(&mut value))
                .desired_rows(8)
                .desired_width(f32::INFINITY),
        )
        .changed()
    {
        send(
            commands,
            AppCommand::UpdateConnectionPluginWorkflowField {
                index,
                field,
                value,
            },
        );
    }
}

fn icon_button(ui: &mut Ui, icon: &str, hover: &str) -> egui::Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(icon).size(16.0)),
        )
    })
    .on_hover_text(hover)
}

fn status_color(status: ConnectionPluginWorkflowStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        ConnectionPluginWorkflowStatus::Ready | ConnectionPluginWorkflowStatus::Valid => {
            tokens.success
        }
        ConnectionPluginWorkflowStatus::Disabled => tokens.text_secondary,
        ConnectionPluginWorkflowStatus::MissingPlugin => tokens.warning,
        ConnectionPluginWorkflowStatus::Invalid | ConnectionPluginWorkflowStatus::Failed => {
            tokens.danger
        }
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
