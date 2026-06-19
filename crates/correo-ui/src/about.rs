use std::time::Duration;

use egui::{Grid, Hyperlink, Image, RichText, ScrollArea, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

const WEBSITE_URL: &str = env!("CARGO_PKG_REPOSITORY");
const EXXETA_URL: &str = "https://exxeta.com";
const ABOUT_ICON_SIZE: f32 = 160.0;
const LOGO_ROLL_OUT_SECONDS: f64 = 1.2;
const LOGO_FADE_IN_SECONDS: f64 = 0.55;

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/about_metadata.rs"));
}

pub fn show(ui: &mut Ui, _tokens: ThemeTokens, i18n: &I18n) -> bool {
    let mut logo_triggered = false;
    ScrollArea::vertical()
        .id_salt("about-content")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            logo_triggered = about_logo(ui);
            ui.add_space(18.0);
            value_row(
                ui,
                i18n.product_name(),
                &format!("v{}", build_info::APP_VERSION.trim_start_matches('v')),
            );
            value_row(ui, &i18n.text("about-license"), env!("CARGO_PKG_LICENSE"));
            ui.add_space(12.0);
            ui.add(Hyperlink::from_label_and_url(
                i18n.text("about-website"),
                WEBSITE_URL,
            ));
            ui.add_space(18.0);
            ui.heading(i18n.text("about-contributors"));
            ui.add(Hyperlink::from_label_and_url("Exxeta", EXXETA_URL));
            ui.add_space(18.0);
            ui.heading(i18n.text("about-open-source-libraries"));
            ui.add_space(8.0);
            open_source_libraries(ui);
        });
    logo_triggered
}

fn about_logo(ui: &mut Ui) -> bool {
    let size = egui::Vec2::splat(ABOUT_ICON_SIZE);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let animation_id = ui.make_persistent_id("about-logo-easter-egg");
    let triggered = response.double_clicked();

    if triggered && !reduce_motion(ui) {
        let now = ui.input(|input| input.time);
        ui.ctx()
            .data_mut(|data| data.insert_temp(animation_id, now));
    }

    let now = ui.input(|input| input.time);
    let started_at = ui.ctx().data_mut(|data| data.get_temp::<f64>(animation_id));
    let Some(started_at) = started_at else {
        paint_logo(ui, rect, 0.0, 1.0);
        return triggered;
    };

    let elapsed = now - started_at;
    let total_seconds = LOGO_ROLL_OUT_SECONDS + LOGO_FADE_IN_SECONDS;
    if elapsed >= total_seconds {
        paint_logo(ui, rect, 0.0, 1.0);
        return triggered;
    }

    if elapsed < LOGO_ROLL_OUT_SECONDS {
        let progress = ease_out_quart((elapsed / LOGO_ROLL_OUT_SECONDS) as f32);
        let offset = (ui.clip_rect().right() - rect.left() + ABOUT_ICON_SIZE) * progress;
        let rotation = offset / (ABOUT_ICON_SIZE * 0.5);
        paint_logo(ui, rect.translate(egui::vec2(offset, 0.0)), rotation, 1.0);
    } else {
        let fade_elapsed = elapsed - LOGO_ROLL_OUT_SECONDS;
        let alpha = ease_out_quart((fade_elapsed / LOGO_FADE_IN_SECONDS) as f32);
        paint_logo(ui, rect, 0.0, alpha);
    }

    ui.ctx().request_repaint_after(Duration::from_millis(16));
    triggered
}

fn paint_logo(ui: &Ui, rect: egui::Rect, rotation: f32, alpha: f32) {
    Image::new(egui::include_image!("../../../assets/icon.svg"))
        .fit_to_exact_size(egui::Vec2::splat(ABOUT_ICON_SIZE))
        .rotate(rotation, egui::Vec2::splat(0.5))
        .tint(egui::Color32::from_white_alpha(
            (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
        ))
        .paint_at(ui, rect);
}

fn reduce_motion(ui: &Ui) -> bool {
    ui.ctx().style().animation_time <= f32::EPSILON
}

fn ease_out_quart(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(4)
}

fn value_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.label(value);
    });
}

fn open_source_libraries(ui: &mut Ui) {
    Grid::new("about-open-source-libraries")
        .num_columns(2)
        .min_row_height(0.0)
        .spacing([18.0, 0.0])
        .show(ui, |ui| {
            for (name, version) in build_info::OPEN_SOURCE_LIBRARIES {
                ui.label(RichText::new(*name).strong().size(14.0));
                ui.label(RichText::new(*version).size(14.0));
                ui.end_row();
            }
        });
}
