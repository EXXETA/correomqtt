use correo_core::{AppCommand, AppCommandSender, AppSnapshot, BuiltInBrokerStatus, SecretInput};
use egui::{RichText, ScrollArea, TextEdit, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;
use crate::widgets::paint_focus_outline;

pub(crate) fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let broker = &snapshot.built_in_broker;
    ui.label(RichText::new(i18n.text("broker-detail")).color(tokens.text_secondary));
    ui.add_space(12.0);

    ui.horizontal(|ui| {
        ui.label(i18n.text("broker-status"));
        ui.label(
            RichText::new(status_label(broker.status, i18n))
                .strong()
                .color(status_color(broker.status, tokens)),
        );
    });
    ui.add_space(12.0);

    ui.add_enabled_ui(!broker.status.is_running(), |ui| {
        ui.horizontal(|ui| {
            ui.label(i18n.text("broker-port"));
            let mut port = broker.port.clone();
            let response = ui.add_sized([96.0, 24.0], TextEdit::singleline(&mut port));
            paint_focus_outline(ui, &response);
            if response.changed() {
                send(commands, AppCommand::UpdateBuiltInBrokerPort(port));
            }
        });
        ui.add_space(8.0);

        let mut credentials_enabled = broker.credentials_enabled;
        let response = ui.checkbox(
            &mut credentials_enabled,
            i18n.text("broker-credentials-enabled"),
        );
        paint_focus_outline(ui, &response);
        if response.changed() {
            send(
                commands,
                AppCommand::SetBuiltInBrokerCredentialsEnabled(credentials_enabled),
            );
        }

        if credentials_enabled {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(i18n.text("broker-username"));
                let mut username = broker.username.clone();
                let response = ui.add_sized([220.0, 24.0], TextEdit::singleline(&mut username));
                paint_focus_outline(ui, &response);
                if response.changed() {
                    send(commands, AppCommand::UpdateBuiltInBrokerUsername(username));
                }
            });
            ui.horizontal(|ui| {
                ui.label(i18n.text("broker-password"));
                let mut password = broker.password.clone();
                let response = ui.add_sized(
                    [220.0, 24.0],
                    TextEdit::singleline(&mut password).password(true),
                );
                paint_focus_outline(ui, &response);
                if response.changed() {
                    send(
                        commands,
                        AppCommand::UpdateBuiltInBrokerPassword(SecretInput::new(password)),
                    );
                }
            });
        }
    });

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        if broker.status.is_running() {
            let stop = ui.button(i18n.text("broker-stop"));
            paint_focus_outline(ui, &stop);
            if stop.clicked() {
                send(commands, AppCommand::StopBuiltInBroker);
            }
        } else {
            let start = ui.button(i18n.text("broker-start"));
            paint_focus_outline(ui, &start);
            if start.clicked() {
                send(commands, AppCommand::StartBuiltInBroker);
            }
        }

        let clear = ui.button(i18n.text("broker-clear-logs"));
        paint_focus_outline(ui, &clear);
        if clear.clicked() {
            send(commands, AppCommand::ClearBuiltInBrokerLogs);
        }
    });

    ui.add_space(16.0);
    ui.heading(i18n.text("broker-logs"));
    ui.separator();
    let log_height = ui.available_height().max(120.0);
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(log_height)
        .show(ui, |ui| {
            if broker.logs.is_empty() {
                ui.label(RichText::new(i18n.text("broker-no-logs")).color(tokens.text_secondary));
            } else {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.spacing_mut().interact_size.y =
                    ui.text_style_height(&egui::TextStyle::Monospace);
                for entry in &broker.logs {
                    ui.label(
                        RichText::new(format!("{}  {}", entry.timestamp, entry.message))
                            .monospace(),
                    );
                }
            }
        });
}

fn status_label(status: BuiltInBrokerStatus, i18n: &I18n) -> String {
    i18n.text(match status {
        BuiltInBrokerStatus::Stopped => "broker-status-stopped",
        BuiltInBrokerStatus::Starting => "broker-status-starting",
        BuiltInBrokerStatus::Running => "broker-status-running",
        BuiltInBrokerStatus::Stopping => "broker-status-stopping",
        BuiltInBrokerStatus::Error => "broker-status-error",
    })
}

fn status_color(status: BuiltInBrokerStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        BuiltInBrokerStatus::Running => egui::Color32::from_rgb(78, 201, 121),
        BuiltInBrokerStatus::Stopped | BuiltInBrokerStatus::Error => {
            egui::Color32::from_rgb(230, 86, 86)
        }
        BuiltInBrokerStatus::Starting | BuiltInBrokerStatus::Stopping => tokens.accent,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
