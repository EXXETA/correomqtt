use crate::layout;
use crate::widgets::{highlight_fill, tile_table_selected_fill};
use crate::{BuiltinTheme, ColorRgb, ThemeColors, ThemeId, ThemeRegistry, ThemeSelection};
use egui::{Color32, CornerRadius, Stroke, Theme, ThemePreference, Visuals};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub struct ThemeTokens {
    pub window_bg: Color32,
    pub panel_bg: Color32,
    pub panel_raised: Color32,
    pub field_bg: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_disabled: Color32,
    pub accent: Color32,
    pub accent_selected_bg: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub script: Color32,
}

impl ThemeTokens {
    pub fn from_colors(colors: ThemeColors) -> Self {
        Self {
            window_bg: color(colors.window_bg),
            panel_bg: color(colors.panel_bg),
            panel_raised: color(colors.panel_raised),
            field_bg: color(colors.field_bg),
            border: color(colors.border),
            text_primary: color(colors.text_primary),
            text_secondary: color(colors.text_secondary),
            text_disabled: color(colors.text_disabled),
            accent: color(colors.accent),
            accent_selected_bg: color(colors.accent_selected_bg),
            success: color(colors.success),
            warning: color(colors.warning),
            danger: color(colors.danger),
            script: color(colors.script),
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, selection: &ThemeSelection) {
    apply_theme_with_registry(ctx, selection, &ThemeRegistry::builtins());
}

pub fn apply_theme_with_registry(
    ctx: &egui::Context,
    selection: &ThemeSelection,
    registry: &ThemeRegistry,
) {
    let dark_tokens = tokens_for_id(registry, &ThemeId::DARK);
    let light_tokens = tokens_for_id(registry, &ThemeId::LIGHT);
    ctx.set_visuals_of(Theme::Dark, visuals_for(dark_tokens, true));
    ctx.set_visuals_of(Theme::Light, visuals_for(light_tokens, false));
    ctx.all_styles_mut(|style| {
        let tokens = if style.visuals.dark_mode {
            dark_tokens
        } else {
            light_tokens
        };
        let highlight = highlight_fill(tokens);
        let selected = tile_table_selected_fill(tokens);
        style.text_styles = scaled_text_styles();
        style.spacing.item_spacing = egui::vec2(
            layout::CONTROL_PADDING as f32,
            layout::CONTROL_PADDING as f32,
        );
        style.spacing.button_padding = layout::button_padding();
        style.spacing.interact_size.y = style.spacing.interact_size.y.max(layout::CONTROL_HEIGHT);
        style.spacing.menu_margin = layout::control_margin();
        style.visuals.window_corner_radius = CornerRadius::same(layout::CORNER_RADIUS);
        style.visuals.window_stroke = Stroke::NONE;
        for widget in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.corner_radius = CornerRadius::same(layout::CORNER_RADIUS);
            widget.bg_stroke = Stroke::NONE;
            widget.expansion = 0.0;
        }
        style.visuals.widgets.hovered.bg_fill = highlight;
        style.visuals.widgets.hovered.weak_bg_fill = highlight;
        style.visuals.widgets.active.bg_fill = highlight;
        style.visuals.widgets.active.weak_bg_fill = highlight;
        style.visuals.widgets.open.bg_fill = highlight;
        style.visuals.widgets.open.weak_bg_fill = highlight;
        style.visuals.selection.bg_fill = selected;
        style.visuals.selection.stroke =
            Stroke::new(0.0, selected_text_color(style.visuals.dark_mode));
    });
    ctx.set_theme(match selection {
        ThemeSelection::System => ThemePreference::System,
        ThemeSelection::Theme(ThemeId::Builtin(BuiltinTheme::Light)) => ThemePreference::Light,
        ThemeSelection::Theme(_) => ThemePreference::Dark,
    });
}

pub fn tokens(ctx: &egui::Context, selection: &ThemeSelection) -> ThemeTokens {
    tokens_with_registry(ctx, selection, &ThemeRegistry::builtins())
}

pub fn tokens_with_registry(
    ctx: &egui::Context,
    selection: &ThemeSelection,
    registry: &ThemeRegistry,
) -> ThemeTokens {
    let id = match resolved_theme(ctx, selection) {
        Theme::Dark => ThemeId::DARK,
        Theme::Light => ThemeId::LIGHT,
    };
    tokens_for_id(registry, &id)
}

pub fn static_tokens(selection: &ThemeSelection) -> ThemeTokens {
    static_tokens_with_registry(selection, &ThemeRegistry::builtins())
}

pub fn static_tokens_with_registry(
    selection: &ThemeSelection,
    registry: &ThemeRegistry,
) -> ThemeTokens {
    match selection {
        ThemeSelection::Theme(id) => tokens_for_id(registry, id),
        ThemeSelection::System => tokens_for_id(registry, &ThemeId::DARK),
    }
}

fn scaled_text_styles() -> BTreeMap<egui::TextStyle, egui::FontId> {
    let mut text_styles = egui::Style::default().text_styles;
    for font in text_styles.values_mut() {
        font.size *= layout::FONT_SIZE_SCALE;
    }
    text_styles
}

fn resolved_theme(ctx: &egui::Context, selection: &ThemeSelection) -> Theme {
    match selection {
        ThemeSelection::Theme(ThemeId::Builtin(BuiltinTheme::Light)) => Theme::Light,
        ThemeSelection::Theme(_) => Theme::Dark,
        ThemeSelection::System => ctx.system_theme().unwrap_or(Theme::Dark),
    }
}

fn tokens_for_id(registry: &ThemeRegistry, id: &ThemeId) -> ThemeTokens {
    let fallback = match id {
        ThemeId::Builtin(BuiltinTheme::Light) => ThemeId::LIGHT,
        _ => ThemeId::DARK,
    };
    let colors = registry
        .resolve(id)
        .or_else(|| registry.resolve(&fallback))
        .or_else(|| registry.resolve(&ThemeId::DARK))
        .expect("built-in themes are always registered")
        .colors;
    ThemeTokens::from_colors(colors)
}

fn visuals_for(tokens: ThemeTokens, dark_mode: bool) -> Visuals {
    let mut visuals = if dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.dark_mode = dark_mode;
    visuals.window_fill = popup_fill(tokens, dark_mode);
    visuals.panel_fill = tokens.panel_bg;
    visuals.extreme_bg_color = tokens.field_bg;
    visuals.faint_bg_color = tokens.panel_raised;
    visuals.code_bg_color = tokens.field_bg;
    visuals.override_text_color = None;
    visuals.warn_fg_color = tokens.warning;
    visuals.error_fg_color = tokens.danger;
    visuals.hyperlink_color = tokens.accent;
    visuals.selection.bg_fill = tile_table_selected_fill(tokens);
    visuals.selection.stroke = Stroke::new(0.0, selected_text_color(dark_mode));
    visuals.window_stroke = Stroke::NONE;
    visuals.widgets.noninteractive.bg_fill = tokens.panel_bg;
    visuals.widgets.noninteractive.weak_bg_fill = tokens.window_bg;
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, tokens.text_primary);
    visuals.widgets.inactive.bg_fill = tokens.panel_raised;
    visuals.widgets.inactive.weak_bg_fill = tokens.panel_raised;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, tokens.text_primary);
    let highlight = highlight_fill(tokens);
    visuals.widgets.hovered.bg_fill = highlight;
    visuals.widgets.hovered.weak_bg_fill = highlight;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, tokens.text_primary);
    visuals.widgets.active.bg_fill = highlight;
    visuals.widgets.active.weak_bg_fill = highlight;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.open.bg_fill = highlight;
    visuals.widgets.open.weak_bg_fill = highlight;
    visuals.widgets.open.bg_stroke = Stroke::NONE;
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.expansion = 0.0;
    }
    visuals
}

fn popup_fill(tokens: ThemeTokens, dark_mode: bool) -> Color32 {
    if dark_mode {
        tokens.panel_raised.gamma_multiply(1.02)
    } else {
        Color32::from_rgb(0xE6, 0xEB, 0xF1)
    }
}

fn selected_text_color(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

fn color(color: ColorRgb) -> Color32 {
    Color32::from_rgb(color.r, color.g, color.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_spacing_matches_styling_spec() {
        assert_eq!(layout::control_margin(), egui::Margin::same(8));
        assert_eq!(layout::control_padding(), egui::vec2(8.0, 8.0));
        assert_eq!(layout::button_padding(), egui::vec2(16.0, 8.0));
    }

    #[test]
    fn text_styles_are_scaled_from_egui_defaults() {
        let defaults = egui::Style::default().text_styles;
        let scaled = scaled_text_styles();

        for (style, default_font) in defaults {
            let scaled_font = scaled
                .get(&style)
                .expect("scaled text styles should preserve default styles");
            assert_eq!(scaled_font.family, default_font.family);
            assert!(
                (scaled_font.size - default_font.size * layout::FONT_SIZE_SCALE).abs()
                    < f32::EPSILON
            );
        }
    }
}
