use std::hash::Hash;

const TILE_HOVER_SECONDS: f32 = 0.12;
const TILE_SELECT_SECONDS: f32 = 0.18;
const FLYOUT_SECONDS: f32 = 0.22;
const FLYOUT_OFFSET: f32 = 28.0;

pub(crate) fn apply_preference(ctx: &egui::Context, reduce_motion: bool) {
    if !reduce_motion {
        return;
    }

    ctx.style_mut(|style| {
        style.animation_time = 0.0;
        style.scroll_animation = egui::style::ScrollAnimation::none();
    });
}

pub(crate) fn tile_fill(
    ui: &egui::Ui,
    id_source: impl Hash,
    base: egui::Color32,
    hover: egui::Color32,
    selected: egui::Color32,
    hovered: bool,
    is_selected: bool,
) -> egui::Color32 {
    let id = ui.make_persistent_id(("tile-motion", id_source));
    let hover_t = bool_progress(ui.ctx(), id.with("hover"), hovered, TILE_HOVER_SECONDS);
    let selected_t = bool_progress(
        ui.ctx(),
        id.with("selected"),
        is_selected,
        TILE_SELECT_SECONDS,
    );

    lerp_color(lerp_color(base, selected, selected_t), hover, hover_t)
}

pub(crate) fn flyout_progress(
    ctx: &egui::Context,
    id_source: impl Hash,
    open: bool,
) -> Option<f32> {
    let progress = bool_progress(
        ctx,
        egui::Id::new(("flyout-motion", id_source)),
        open,
        FLYOUT_SECONDS,
    );
    if open || progress > 0.01 {
        Some(progress)
    } else {
        None
    }
}

pub(crate) fn flyout_panel_rect(
    scrim_rect: egui::Rect,
    panel_width: f32,
    progress: f32,
) -> egui::Rect {
    let width = panel_width.min(scrim_rect.width());
    let rect = egui::Rect::from_min_size(
        scrim_rect.left_top(),
        egui::vec2(width, scrim_rect.height()),
    );
    let offset = FLYOUT_OFFSET.min(width * 0.12) * (1.0 - progress);
    rect.translate(egui::vec2(-offset, 0.0))
}

pub(crate) fn scrim_color(max_alpha: u8, progress: f32) -> egui::Color32 {
    egui::Color32::from_black_alpha((f32::from(max_alpha) * progress).round() as u8)
}

pub(crate) fn content_opacity(progress: f32) -> f32 {
    progress.clamp(0.0, 1.0)
}

fn bool_progress(ctx: &egui::Context, id: egui::Id, target: bool, seconds: f32) -> f32 {
    if reduce_motion(ctx) {
        if target {
            1.0
        } else {
            0.0
        }
    } else {
        ctx.animate_bool_with_time_and_easing(id, target, seconds, ease_out_quart)
    }
}

fn reduce_motion(ctx: &egui::Context) -> bool {
    ctx.style().animation_time <= f32::EPSILON
}

fn ease_out_quart(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(4)
}

fn lerp_color(from: egui::Color32, to: egui::Color32, t: f32) -> egui::Color32 {
    let [from_r, from_g, from_b, from_a] = from.to_array();
    let [to_r, to_g, to_b, to_a] = to.to_array();
    egui::Color32::from_rgba_unmultiplied(
        lerp_channel(from_r, to_r, t),
        lerp_channel(from_g, to_g, t),
        lerp_channel(from_b, to_b, t),
        lerp_channel(from_a, to_a, t),
    )
}

fn lerp_channel(from: u8, to: u8, t: f32) -> u8 {
    (f32::from(from) + (f32::from(to) - f32::from(from)) * t.clamp(0.0, 1.0)).round() as u8
}
