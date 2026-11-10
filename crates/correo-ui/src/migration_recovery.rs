use correo_core::{
    AppCommandSender, MigrationFailureStage, MigrationPasswordError, MigrationRecoveryCommand,
    MigrationRecoveryCompletion, MigrationRecoverySnapshot, MigrationRecoveryState,
    MigrationRecoveryStep,
};
use egui::{RichText, ScrollArea, TextEdit, Ui, Window};

use crate::theme::ThemeTokens;

#[path = "migration_recovery/actions.rs"]
mod actions;
use actions::{action_bar, button, handle_keyboard, send, send_submit_password};

pub fn top_bar(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot) {
    ui.horizontal_centered(|ui| {
        ui.label(
            RichText::new("CorreoMQTT Beta Migration Recovery")
                .strong()
                .size(16.0),
        );
        ui.separator();
        ui.label(state_label(snapshot.state));
    });
}

pub fn context_panel(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot, tokens: ThemeTokens) {
    ui.heading("Legacy migration");
    ui.separator();
    label_value(
        ui,
        "Detected path",
        snapshot.legacy_path.as_deref().unwrap_or("Not detected"),
    );
    label_value(ui, "Backup", backup_status(snapshot));
    ui.separator();
    label_value(ui, "Connections", &snapshot.counts.connections.to_string());
    label_value(ui, "Histories", &snapshot.counts.histories.to_string());
    label_value(ui, "Scripts", &snapshot.counts.scripts.to_string());
    label_value(
        ui,
        "Plugin artifacts ignored",
        &snapshot.counts.plugin_artifacts_ignored.to_string(),
    );
    label_value(ui, "Warnings", &snapshot.warning_count().to_string());
    if snapshot.counts.skipped_secrets > 0 {
        label_value(
            ui,
            "Skipped secrets",
            &snapshot.counts.skipped_secrets.to_string(),
        );
    }
    ui.separator();
    for diagnostic in snapshot.diagnostics.iter().rev().take(4) {
        ui.label(RichText::new(&diagnostic.message).color(tokens.text_secondary));
    }
}

pub fn show(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let mut legacy_master_password = legacy_master_password(ui);
    handle_keyboard(ui, snapshot, commands, &mut legacy_master_password);
    let scroll_height = (ui.available_height() - 48.0).max(180.0);
    ScrollArea::vertical()
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            stepper(ui, snapshot, tokens);
            ui.separator();
            match snapshot.state {
                MigrationRecoveryState::NotDetected => no_legacy_data(ui, tokens),
                MigrationRecoveryState::Detecting => detecting(ui, tokens),
                MigrationRecoveryState::NeedsDecision => detection(ui, snapshot, tokens),
                MigrationRecoveryState::CreatingBackup => creating_backup(ui, snapshot, tokens),
                MigrationRecoveryState::NeedsPassword => {
                    unlock(ui, snapshot, tokens, commands, &mut legacy_master_password)
                }
                MigrationRecoveryState::Reviewing => review(ui, snapshot, tokens, commands),
                MigrationRecoveryState::Applying => applying(ui, snapshot, tokens),
                MigrationRecoveryState::Complete => complete(ui, snapshot, tokens),
                MigrationRecoveryState::Failed => failed(ui, snapshot, tokens),
                MigrationRecoveryState::RestoreConfirm => restore_confirm(ui, snapshot, tokens),
                MigrationRecoveryState::Restoring => restoring(ui, tokens),
            }
        });
    ui.separator();
    action_bar(ui, snapshot, commands, &mut legacy_master_password);
    store_legacy_master_password(ui, snapshot, legacy_master_password);
    empty_profile_confirmation(ui, snapshot, commands);
}

fn stepper(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot, tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        for step in MigrationRecoveryStep::ALL {
            let selected = snapshot.active_step() == step;
            let text = if selected {
                RichText::new(step.label()).strong().color(tokens.accent)
            } else {
                RichText::new(step.label()).color(tokens.text_secondary)
            };
            ui.label(text);
            if step != MigrationRecoveryStep::Complete {
                ui.label(RichText::new(">").color(tokens.text_secondary));
            }
        }
    });
}

fn no_legacy_data(ui: &mut Ui, tokens: ThemeTokens) {
    ui.heading("No legacy data detected");
    ui.label(RichText::new("Connections workspace is available.").color(tokens.text_secondary));
}

fn detecting(ui: &mut Ui, tokens: ThemeTokens) {
    ui.heading("Detecting legacy data...");
    ui.label(
        RichText::new("Checking supported Java CorreoMQTT paths.").color(tokens.text_secondary),
    );
}

fn detection(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot, tokens: ThemeTokens) {
    ui.heading("Legacy CorreoMQTT data found");
    let path = snapshot
        .legacy_path
        .as_deref()
        .unwrap_or("the legacy profile");
    ui.label(format!(
        "CorreoMQTT Beta found data from the Java version at {path}. A backup will be created before anything is changed."
    ));
    ui.add_space(8.0);
    plugin_note(ui, tokens);
}

fn creating_backup(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot, tokens: ThemeTokens) {
    ui.heading("Legacy CorreoMQTT data found");
    ui.label("Creating backup before migration...");
    ui.label(RichText::new(backup_status(snapshot)).color(tokens.text_secondary));
}

fn unlock(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    legacy_master_password: &mut String,
) {
    ui.heading("Unlock legacy secrets");
    ui.label("Enter the legacy master password to import saved connection secrets into the OS keyring. The password is not stored.");
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_sized(
            [260.0, crate::theme::CONTROL_HEIGHT],
            crate::widgets::padded_text_edit(TextEdit::singleline(legacy_master_password))
                .password(true)
                .hint_text("Legacy master password"),
        );
        if ui.button("Unlock secrets").clicked() {
            send_submit_password(commands, legacy_master_password);
        }
    });
    if let Some(error) = snapshot.password_error {
        ui.label(RichText::new(password_error(error)).color(tokens.danger));
    }
}

fn review(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading("Review migration");
    ui.label(
        "Review warnings before applying migration. Secrets and key material are never shown.",
    );
    if snapshot.secrets_skipped || snapshot.counts.skipped_secrets > 0 {
        let count = snapshot.counts.skipped_secrets.max(1);
        ui.label(
            RichText::new(format!(
                "{count} connection secret(s) were skipped during migration."
            ))
            .color(tokens.warning),
        );
    }
    ui.add_space(8.0);
    for row in &snapshot.rows {
        ui.horizontal_wrapped(|ui| {
            let mut selected = row.selected;
            if crate::widgets::checkbox(ui, &mut selected, row.task.label()).changed() {
                send(
                    commands,
                    MigrationRecoveryCommand::SelectMigrationItem {
                        item_id: row.id.clone(),
                        selected,
                    },
                );
            }
            ui.label(RichText::new(&row.label).strong());
            ui.label(RichText::new(&row.detail).color(tokens.text_secondary));
        });
        if let Some(warning) = &row.warning {
            ui.label(RichText::new(warning).color(tokens.warning));
        }
    }
    if !snapshot.warnings.is_empty() {
        ui.separator();
        for warning in &snapshot.warnings {
            ui.label(RichText::new(&warning.message).color(tokens.warning));
        }
    }
    plugin_note(ui, tokens);
}

fn applying(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot, tokens: ThemeTokens) {
    ui.heading("Migrating legacy data...");
    ui.label(
        "Keep CorreoMQTT open until migration finishes. Backup is available if recovery is needed.",
    );
    if let Some(stage) = snapshot.current_stage {
        ui.label(RichText::new(stage.label()).color(tokens.accent));
    }
}

fn complete(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot, tokens: ThemeTokens) {
    let completion = snapshot
        .completion
        .unwrap_or(MigrationRecoveryCompletion::Success);
    match completion {
        MigrationRecoveryCompletion::Success => {
            ui.heading("Migration complete");
            ui.label(format!(
                "{} connection profile(s), {} history file(s), and {} script(s) migrated. Review diagnostics for warnings.",
                snapshot.counts.connections, snapshot.counts.histories, snapshot.counts.scripts
            ));
        }
        MigrationRecoveryCompletion::PartialSuccess => {
            ui.heading("Migration completed with warnings");
            ui.label("Some legacy settings could not be mapped. CorreoMQTT preserved the backup and recorded details in Diagnostics.");
        }
        MigrationRecoveryCompletion::RestoreSuccess => {
            ui.heading("Backup restored");
            ui.label("CorreoMQTT restored the backup. Restart may be required for all settings to reload.");
        }
    }
    if snapshot.warning_count() > 0 {
        ui.label(RichText::new("Diagnostics contain migration warnings.").color(tokens.warning));
    }
}

fn failed(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot, tokens: ThemeTokens) {
    let Some(failure) = &snapshot.failure else {
        ui.heading("Migration failed before changes were written");
        ui.label("CorreoMQTT did not change your beta data. You can retry migration or start with an empty beta profile.");
        return;
    };
    match failure.stage {
        MigrationFailureStage::Backup => {
            ui.heading("Backup failed");
            ui.label("CorreoMQTT did not change your data. Choose another backup location or start with an empty beta profile.");
        }
        MigrationFailureStage::BeforeWrite => {
            ui.heading("Migration failed before changes were written");
            ui.label("CorreoMQTT did not change your beta data. You can retry migration or start with an empty beta profile.");
        }
        MigrationFailureStage::AfterWrite => {
            ui.heading("Migration stopped after changes started");
            ui.label("A backup was created before migration. Restore the backup to return to the previous beta data, or keep the partial migration and review diagnostics.");
        }
        MigrationFailureStage::Restore => {
            ui.heading("Restore failed");
            ui.label("The backup was not restored. Current beta data was left unchanged; review diagnostics for details.");
        }
    }
    ui.label(RichText::new(&failure.message).color(tokens.danger));
}

fn restore_confirm(ui: &mut Ui, snapshot: &MigrationRecoverySnapshot, tokens: ThemeTokens) {
    ui.heading("Restore backup?");
    let backup = snapshot.backup_name.as_deref().unwrap_or("selected backup");
    ui.label(format!(
        "Restoring replaces the current beta profile with backup {backup}. The backup itself will not be deleted."
    ));
    ui.label(RichText::new("Confirm before restore starts.").color(tokens.warning));
}

fn restoring(ui: &mut Ui, tokens: ThemeTokens) {
    ui.heading("Restoring backup...");
    ui.label(
        RichText::new("Current beta profile is protected until restore succeeds.")
            .color(tokens.text_secondary),
    );
}

fn empty_profile_confirmation(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    commands: &AppCommandSender,
) {
    if !snapshot.empty_profile_confirmation_open {
        return;
    }
    Window::new("Start without migrating?")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            let path = snapshot.legacy_path.as_deref().unwrap_or("the legacy path");
            ui.label(format!(
                "Your legacy data will stay in {path}. You can return to migration from Settings > Data."
            ));
            ui.horizontal(|ui| {
                button(
                    ui,
                    "Start empty profile",
                    commands,
                    MigrationRecoveryCommand::ConfirmStartEmptyProfile,
                );
                button(ui, "Cancel", commands, MigrationRecoveryCommand::CancelEmptyProfile);
            });
        });
}

fn plugin_note(ui: &mut Ui, tokens: ThemeTokens) {
    ui.separator();
    ui.label(
        RichText::new("Java plugins were reinitialized")
            .strong()
            .color(tokens.warning),
    );
    ui.label(MigrationRecoverySnapshot::plugin_compatibility_body());
    ui.label(MigrationRecoverySnapshot::plugin_replacement_body());
}

fn password_error(error: MigrationPasswordError) -> &'static str {
    match error {
        MigrationPasswordError::WrongPassword => {
            "That password did not unlock the legacy secrets. The file is still selected; you can retry or skip secrets for now."
        }
        MigrationPasswordError::UnsupportedEncryption => {
            "This password file uses an unsupported encryption format. Connections can still be migrated, but secrets must be restored manually."
        }
    }
}

fn label_value(ui: &mut Ui, label: &str, value: &str) {
    ui.label(RichText::new(label).strong());
    ui.label(value);
}

fn legacy_password_id(ui: &Ui) -> egui::Id {
    ui.make_persistent_id("legacy-master-password")
}

fn legacy_master_password(ui: &Ui) -> String {
    ui.ctx()
        .data(|data| data.get_temp::<String>(legacy_password_id(ui)))
        .unwrap_or_default()
}

fn store_legacy_master_password(
    ui: &Ui,
    snapshot: &MigrationRecoverySnapshot,
    legacy_master_password: String,
) {
    let id = legacy_password_id(ui);
    ui.ctx().data_mut(|data| {
        if snapshot.state == MigrationRecoveryState::NeedsPassword
            && !legacy_master_password.is_empty()
        {
            data.insert_temp(id, legacy_master_password);
        } else {
            data.remove_temp::<String>(id);
        }
    });
}

fn backup_status(snapshot: &MigrationRecoverySnapshot) -> &str {
    if snapshot.backup_status.is_empty() {
        "Backup not created yet"
    } else {
        &snapshot.backup_status
    }
}

fn state_label(state: MigrationRecoveryState) -> &'static str {
    match state {
        MigrationRecoveryState::NotDetected => "Not detected",
        MigrationRecoveryState::Detecting => "Detecting",
        MigrationRecoveryState::NeedsDecision => "Legacy data found",
        MigrationRecoveryState::CreatingBackup => "Creating backup",
        MigrationRecoveryState::NeedsPassword => "Unlock secrets",
        MigrationRecoveryState::Reviewing => "Review migration",
        MigrationRecoveryState::Applying => "Applying",
        MigrationRecoveryState::Complete => "Complete",
        MigrationRecoveryState::Failed => "Failed",
        MigrationRecoveryState::RestoreConfirm => "Restore confirmation",
        MigrationRecoveryState::Restoring => "Restoring",
    }
}
