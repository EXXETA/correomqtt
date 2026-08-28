use correo_core::{
    AppCommand, AppCommandSender, MigrationRecoveryCommand, MigrationRecoverySnapshot,
    MigrationRecoveryState, SecretInput,
};
use egui::{Button, Ui};

const BUTTON_WIDTH: f32 = 190.0;
const BUTTON_HEIGHT: f32 = crate::theme::CONTROL_HEIGHT;

pub(super) fn action_bar(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    commands: &AppCommandSender,
    legacy_master_password: &mut String,
) {
    ui.horizontal_wrapped(|ui| {
        ui.add_space((ui.available_width() - action_width(snapshot)).max(0.0));
        actions(ui, snapshot, commands, legacy_master_password);
    });
}

fn actions(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    commands: &AppCommandSender,
    legacy_master_password: &mut String,
) {
    match snapshot.state {
        MigrationRecoveryState::NeedsDecision => {
            button(
                ui,
                "Start empty beta profile",
                commands,
                MigrationRecoveryCommand::StartEmptyProfile,
            );
            button(
                ui,
                "Create backup and migrate",
                commands,
                MigrationRecoveryCommand::ChooseMigrate,
            );
        }
        MigrationRecoveryState::CreatingBackup
        | MigrationRecoveryState::Applying
        | MigrationRecoveryState::Restoring => {
            ui.add_enabled(false, Button::new("Migration in progress"));
        }
        MigrationRecoveryState::NeedsPassword => {
            button(ui, "Back", commands, MigrationRecoveryCommand::Retry);
            button(
                ui,
                "Skip secrets for now",
                commands,
                MigrationRecoveryCommand::SkipSecrets,
            );
            if ui
                .add_sized([BUTTON_WIDTH, BUTTON_HEIGHT], Button::new("Unlock secrets"))
                .clicked()
            {
                send_submit_password(commands, legacy_master_password);
            }
        }
        MigrationRecoveryState::Reviewing => {
            button(ui, "Back", commands, MigrationRecoveryCommand::Retry);
            if ui
                .add_enabled(
                    snapshot.selected_count() > 0,
                    Button::new("Migrate selected")
                        .min_size(egui::vec2(BUTTON_WIDTH, BUTTON_HEIGHT)),
                )
                .clicked()
            {
                send(commands, MigrationRecoveryCommand::ApplyMigration);
            }
        }
        MigrationRecoveryState::Complete => {
            button(
                ui,
                "Open Connections",
                commands,
                MigrationRecoveryCommand::OpenConnections,
            );
        }
        MigrationRecoveryState::Failed => {
            button(
                ui,
                "Start empty beta profile",
                commands,
                MigrationRecoveryCommand::StartEmptyProfile,
            );
            button(ui, "Retry", commands, MigrationRecoveryCommand::Retry);
            if snapshot.backup_name.is_some() {
                button(
                    ui,
                    "Restore backup...",
                    commands,
                    MigrationRecoveryCommand::RequestRestoreBackup,
                );
            }
        }
        MigrationRecoveryState::RestoreConfirm => {
            button(
                ui,
                "Cancel",
                commands,
                MigrationRecoveryCommand::CancelRestoreBackup,
            );
            button(
                ui,
                "Restore backup",
                commands,
                MigrationRecoveryCommand::ConfirmRestoreBackup,
            );
        }
        MigrationRecoveryState::NotDetected | MigrationRecoveryState::Detecting => {}
    }
}

fn action_width(snapshot: &MigrationRecoverySnapshot) -> f32 {
    let count = match snapshot.state {
        MigrationRecoveryState::NeedsDecision
        | MigrationRecoveryState::Reviewing
        | MigrationRecoveryState::RestoreConfirm => 2,
        MigrationRecoveryState::Complete => 1,
        MigrationRecoveryState::NeedsPassword => 3,
        MigrationRecoveryState::Failed if snapshot.backup_name.is_some() => 3,
        MigrationRecoveryState::Failed => 2,
        MigrationRecoveryState::CreatingBackup
        | MigrationRecoveryState::Applying
        | MigrationRecoveryState::Restoring => 1,
        MigrationRecoveryState::NotDetected | MigrationRecoveryState::Detecting => 0,
    };
    if count == 0 {
        0.0
    } else {
        count as f32 * BUTTON_WIDTH + (count - 1) as f32 * 8.0
    }
}

pub(super) fn handle_keyboard(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    commands: &AppCommandSender,
    legacy_master_password: &mut String,
) {
    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        if snapshot.empty_profile_confirmation_open {
            send(commands, MigrationRecoveryCommand::CancelEmptyProfile);
        } else if snapshot.state == MigrationRecoveryState::RestoreConfirm {
            send(commands, MigrationRecoveryCommand::CancelRestoreBackup);
        }
    }
    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        if snapshot.state == MigrationRecoveryState::NeedsPassword {
            send_submit_password(commands, legacy_master_password);
        } else if let Some(command) = primary_command(snapshot) {
            send(commands, command);
        }
    }
}

pub(super) fn button(
    ui: &mut Ui,
    label: &str,
    commands: &AppCommandSender,
    command: MigrationRecoveryCommand,
) {
    if ui
        .add_sized([BUTTON_WIDTH, BUTTON_HEIGHT], Button::new(label))
        .clicked()
    {
        send(commands, command);
    }
}

pub(super) fn send(commands: &AppCommandSender, command: MigrationRecoveryCommand) {
    let _ = commands.send(AppCommand::MigrationRecovery(command));
}

fn primary_command(snapshot: &MigrationRecoverySnapshot) -> Option<MigrationRecoveryCommand> {
    match snapshot.state {
        MigrationRecoveryState::NeedsDecision => Some(MigrationRecoveryCommand::ChooseMigrate),
        MigrationRecoveryState::Reviewing if snapshot.selected_count() > 0 => {
            Some(MigrationRecoveryCommand::ApplyMigration)
        }
        MigrationRecoveryState::Complete => Some(MigrationRecoveryCommand::OpenConnections),
        MigrationRecoveryState::RestoreConfirm => {
            Some(MigrationRecoveryCommand::ConfirmRestoreBackup)
        }
        _ => None,
    }
}

pub(super) fn send_submit_password(commands: &AppCommandSender, password: &mut String) {
    let submitted = std::mem::take(password);
    // Never submit an empty password. The field is cleared on the first
    // submit, so a second Enter/click would otherwise send an empty password
    // whose rejection could arrive after a successful unlock and bounce the UI
    // back to the password screen.
    if submitted.trim().is_empty() {
        return;
    }
    send(
        commands,
        MigrationRecoveryCommand::SubmitPassword {
            password: SecretInput::new(submitted),
        },
    );
}
