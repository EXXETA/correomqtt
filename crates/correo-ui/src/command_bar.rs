use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, GlobalSettingField, SettingsOption, ThemeMode,
};
use correo_style::layout;
use egui::{load::TexturePoll, ComboBox, Image, RichText, Sense, Ui};

use crate::{
    i18n::{language_option_visible, I18n},
    widgets::dotted_focus_outline,
};

const HEADER_LOGO_SIZE: f32 = 34.0;
const HEADER_LOGO_RASTER_SCALE: f32 = 2.0;
const HEADER_CONTROLS_RIGHT_MARGIN: f32 = layout::CONTROL_PADDING as f32 * 2.0 + 6.0;

pub fn command_bar_title(ui: &mut Ui, i18n: &I18n) {
    ui.horizontal_centered(|ui| {
        header_icon(ui);
        ui.label(
            RichText::new(i18n.product_name())
                .strong()
                .size(layout::APP_TITLE_SIZE),
        );
    });
}

pub fn command_bar_controls(
    context: &egui::Context,
    header_rect: egui::Rect,
    snapshot: &AppSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
    klingon_unlocked: bool,
) {
    let width = layout::HEADER_LANGUAGE_SELECTOR_WIDTH
        + layout::HEADER_THEME_SELECTOR_WIDTH
        + layout::CONTROL_PADDING as f32;
    let pos = egui::pos2(
        header_rect.right() - width - HEADER_CONTROLS_RIGHT_MARGIN,
        header_rect.center().y - layout::CONTROL_HEIGHT * 0.5,
    );
    egui::Area::new("correo-header-controls".into())
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(context, |ui| {
            ui.horizontal(|ui| {
                running_scripts_label(ui, &snapshot.scripts);
                language_selector(
                    ui,
                    &snapshot.global_settings.language,
                    &snapshot.global_settings.language_options,
                    commands,
                    i18n,
                    klingon_unlocked,
                );
                theme_selector(ui, &snapshot.theme_mode, commands, i18n);
            });
        });
}

fn header_icon(ui: &mut Ui) {
    let size = egui::Vec2::splat(HEADER_LOGO_SIZE);
    let (rect, _response) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let raster_size = size * HEADER_LOGO_RASTER_SCALE;
    let source = Image::from_bytes(
        "bytes://correo-header-icon-mono.svg",
        include_bytes!("../../../assets/icon_mono.svg"),
    )
    .fit_to_exact_size(raster_size);
    if let Ok(TexturePoll::Ready { texture }) = source.load_for_size(ui.ctx(), raster_size) {
        Image::from_texture(texture)
            .fit_to_exact_size(size)
            .tint(ui.visuals().text_color())
            .paint_at(ui, rect);
    }
}

fn running_scripts_label(ui: &mut Ui, scripts: &correo_core::ScriptSurfaceSnapshot) {
    let running = scripts
        .executions
        .iter()
        .filter(|execution| !execution.status.is_terminal())
        .count();
    if running == 0 {
        return;
    }

    let label = if running == 1 {
        "1 script running".to_owned()
    } else {
        format!("{running} scripts running")
    };
    ui.label(RichText::new(label).size(12.0).weak());
}

fn theme_selector(ui: &mut Ui, current: &ThemeMode, commands: &AppCommandSender, i18n: &I18n) {
    let mut selected = current.clone();
    let response = ComboBox::from_id_salt("theme-mode")
        .selected_text(i18n.theme_label(current))
        .width(layout::HEADER_THEME_SELECTOR_WIDTH)
        .show_ui(ui, |ui| {
            for mode in ThemeMode::ALL {
                ui.selectable_value(&mut selected, mode.clone(), i18n.theme_label(&mode));
            }
        })
        .response;
    if response.has_focus() {
        dotted_focus_outline(ui, response.rect);
    }
    if selected != *current {
        let _ = commands.send(AppCommand::SetThemeMode(selected));
        let _ = commands.send(AppCommand::SaveGlobalSettings);
    }
}

fn language_selector(
    ui: &mut Ui,
    current: &str,
    options: &[SettingsOption],
    commands: &AppCommandSender,
    i18n: &I18n,
    klingon_unlocked: bool,
) {
    let mut selected = current.to_owned();
    let response = ComboBox::from_id_salt("header-language")
        .selected_text(language_label(current, options, i18n))
        .width(layout::HEADER_LANGUAGE_SELECTOR_WIDTH)
        .show_ui(ui, |ui| {
            for option in options {
                if !language_option_visible(&option.id, current, klingon_unlocked) {
                    continue;
                }
                let label = i18n.language_menu_option_label(&option.id, &option.label, current);
                ui.selectable_value(&mut selected, option.id.clone(), label);
            }
        })
        .response;
    if response.has_focus() {
        dotted_focus_outline(ui, response.rect);
    }
    if selected != current {
        let _ = commands.send(AppCommand::UpdateGlobalSetting {
            field: GlobalSettingField::Language,
            value: selected,
        });
        let _ = commands.send(AppCommand::SaveGlobalSettings);
    }
}

fn language_label(current: &str, options: &[SettingsOption], i18n: &I18n) -> String {
    options
        .iter()
        .find(|option| option.id == current)
        .map(|option| i18n.language_menu_option_label(&option.id, &option.label, current))
        .unwrap_or_else(|| current.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_label_uses_available_option_labels() {
        let options = vec![
            SettingsOption {
                id: "system".to_owned(),
                label: "System".to_owned(),
            },
            SettingsOption {
                id: "de_DE".to_owned(),
                label: "Deutsch".to_owned(),
            },
        ];
        let i18n = I18n::new("en_US");

        assert_eq!(language_label("system", &options, &i18n), "System");
        assert_eq!(language_label("de_DE", &options, &i18n), "Deutsch");
        assert_eq!(language_label("custom", &options, &i18n), "custom");
    }

    #[test]
    fn app_title_font_size_is_scaled_up_for_header() {
        assert_eq!(layout::APP_TITLE_SIZE, 28.0);
        assert_eq!(layout::APP_TITLE_SIZE, layout::APP_TITLE_BASE_SIZE * 1.75);
    }
}
