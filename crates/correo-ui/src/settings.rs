use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, GlobalSettingField, GlobalSettingFlag,
    GlobalSettingsSnapshot, SettingsOption,
};
use correo_style::layout;
use egui::{Button, ComboBox, RichText, ScrollArea, TextEdit, Ui};
use egui_phosphor::regular;

use crate::i18n::{language_option_visible, I18n};
use crate::theme::{ThemeTokens, CONTROL_HEIGHT};
use crate::widgets::{
    checkbox, padded_text_edit, square_icon_button_size, with_icon_button_padding,
};

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    klingon_unlocked: bool,
) {
    let settings = &snapshot.global_settings;
    ui.heading(i18n.text("settings-header"));
    ui.add_space(12.0);

    ScrollArea::vertical()
        .id_salt("global-settings-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Appearance),
                tokens,
                |ui| {
                    appearance(ui, settings, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Search),
                tokens,
                |ui| {
                    search(ui, settings, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Keyring),
                tokens,
                |ui| {
                    keyring(ui, settings, tokens, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Updates),
                tokens,
                |ui| {
                    updates(ui, settings, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Plugins),
                tokens,
                |ui| {
                    plugins(ui, settings, tokens, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Language),
                tokens,
                |ui| {
                    language(ui, settings, commands, i18n, klingon_unlocked);
                },
            );
        });
}

fn language(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
    klingon_unlocked: bool,
) {
    row(ui, &i18n.text("settings-language"), |ui| {
        option_combo(
            ui,
            "settings-language",
            &settings.language,
            &settings.language_options,
            |value| AppCommand::UpdateGlobalSetting {
                field: GlobalSettingField::Language,
                value,
            },
            commands,
            i18n,
            Some(klingon_unlocked),
        );
    });
}

fn section(ui: &mut Ui, title: String, tokens: ThemeTokens, add: impl FnOnce(&mut Ui)) {
    ui.add_space(14.0);
    ui.label(
        RichText::new(title)
            .strong()
            .size(18.0)
            .color(tokens.text_primary),
    );
    ui.add_space(8.0);
    add(ui);
    ui.add_space(14.0);
}

fn appearance(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    checkbox_flag(
        ui,
        &i18n.text("settings-reduce-motion"),
        settings.reduce_motion,
        GlobalSettingFlag::ReduceMotion,
        commands,
    );
}

fn search(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    checkbox_flag(
        ui,
        &i18n.text("settings-use-regex"),
        settings.search_use_regex,
        GlobalSettingFlag::UseRegexForSearch,
        commands,
    );
    checkbox_flag(
        ui,
        &i18n.text("settings-ignore-case"),
        settings.search_ignore_case,
        GlobalSettingFlag::UseIgnoreCase,
        commands,
    );
}

fn keyring(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    row(ui, &i18n.text("settings-backend"), |ui| {
        option_combo(
            ui,
            "settings-keyring",
            &settings.keyring_backend,
            &settings.keyring_options,
            |value| AppCommand::UpdateGlobalSetting {
                field: GlobalSettingField::KeyringBackend,
                value,
            },
            commands,
            i18n,
            None,
        );
    });
}

fn updates(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    checkbox_flag(
        ui,
        &i18n.text("settings-updates"),
        settings.update_checks_enabled,
        GlobalSettingFlag::SearchUpdates,
        commands,
    );
    row(ui, &i18n.text("settings-last-update-check"), |ui| {
        ui.label(&settings.last_update_check);
    });
}

fn plugins(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    checkbox_flag(
        ui,
        &i18n.text("settings-use-default-repository"),
        settings.use_default_plugin_repository,
        GlobalSettingFlag::UseDefaultPluginRepository,
        commands,
    );
    checkbox_flag(
        ui,
        &i18n.text("settings-install-bundled-plugins"),
        settings.install_bundled_plugins,
        GlobalSettingFlag::InstallBundledPlugins,
        commands,
    );
    if settings.plugin_repositories.is_empty() {
        row(ui, &i18n.text("settings-plugin-repositories"), |ui| {
            add_repository_button(ui, commands, i18n);
        });
    } else {
        for (index, repository) in settings.plugin_repositories.iter().enumerate() {
            repository_row(ui, index, &repository.url, commands, i18n);
        }
        button_row(ui, |ui| {
            add_repository_button(ui, commands, i18n);
        });
    }
}

fn row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(CONTROL_HEIGHT);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(layout::SETTINGS_LABEL_WIDTH, CONTROL_HEIGHT),
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

fn button_row(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    row(ui, "", add);
}

fn repository_row(
    ui: &mut Ui,
    index: usize,
    current_url: &str,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let label = if index == 0 {
        i18n.text("settings-plugin-repositories")
    } else {
        String::new()
    };
    row(ui, &label, |ui| {
        let mut url = current_url.to_owned();
        let response = ui.add_sized(
            [layout::SETTINGS_CONTROL_WIDTH, CONTROL_HEIGHT],
            padded_text_edit(TextEdit::singleline(&mut url)),
        );
        if response.changed() {
            send_and_save(commands, AppCommand::UpdatePluginRepository { index, url });
        }
        if icon_button(
            ui,
            regular::MINUS,
            i18n.text("settings-remove-plugin-repository"),
        )
        .clicked()
        {
            send_and_save(commands, AppCommand::RemovePluginRepository { index });
        }
    });
}

fn add_repository_button(ui: &mut Ui, commands: &AppCommandSender, i18n: &I18n) {
    if icon_button(
        ui,
        regular::PLUS,
        i18n.text("settings-add-plugin-repository"),
    )
    .clicked()
    {
        send_and_save(commands, AppCommand::AddPluginRepository);
    }
}

fn icon_button(ui: &mut Ui, icon: &str, hover_text: String) -> egui::Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(icon).size(16.0)),
        )
    })
    .on_hover_text(hover_text)
}

fn checkbox_flag(
    ui: &mut Ui,
    label: &str,
    current: bool,
    flag: GlobalSettingFlag,
    commands: &AppCommandSender,
) {
    let mut enabled = current;
    row(ui, label, |ui| {
        if checkbox(ui, &mut enabled, "").changed() {
            send_and_save(commands, AppCommand::SetGlobalSettingFlag { flag, enabled });
        }
    });
}

fn option_combo(
    ui: &mut Ui,
    id: &str,
    current: &str,
    options: &[SettingsOption],
    command: impl FnOnce(String) -> AppCommand,
    commands: &AppCommandSender,
    i18n: &I18n,
    language_klingon_unlocked: Option<bool>,
) {
    let mut selected = current.to_owned();
    let selected_text = if language_klingon_unlocked.is_some() {
        language_option_label(current, options, i18n)
    } else {
        option_label(current, options, i18n)
    };
    ComboBox::from_id_salt(id)
        .selected_text(selected_text)
        .width(layout::SETTINGS_COMBO_WIDTH)
        .show_ui(ui, |ui| {
            for option in options {
                if let Some(klingon_unlocked) = language_klingon_unlocked {
                    if !language_option_visible(&option.id, current, klingon_unlocked) {
                        continue;
                    }
                }
                let label = if language_klingon_unlocked.is_some() {
                    i18n.language_menu_option_label(&option.id, &option.label, current)
                } else {
                    i18n.language_option_label(&option.id, &option.label)
                };
                ui.selectable_value(&mut selected, option.id.clone(), label);
            }
        });
    if selected != current {
        send_and_save(commands, command(selected));
    }
}

fn option_label(current: &str, options: &[SettingsOption], i18n: &I18n) -> String {
    options
        .iter()
        .find(|option| option.id == current)
        .map(|option| i18n.language_option_label(&option.id, &option.label))
        .unwrap_or_else(|| current.to_owned())
}

fn language_option_label(current: &str, options: &[SettingsOption], i18n: &I18n) -> String {
    options
        .iter()
        .find(|option| option.id == current)
        .map(|option| i18n.language_menu_option_label(&option.id, &option.label, current))
        .unwrap_or_else(|| current.to_owned())
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}

fn send_and_save(commands: &AppCommandSender, command: AppCommand) {
    send(commands, command);
    send(commands, AppCommand::SaveGlobalSettings);
}
