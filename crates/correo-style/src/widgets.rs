use std::sync::Arc;

use crate::{layout, ThemeTokens};
use egui::{
    pos2, vec2, FontId, Galley, Response, Sense, TextEdit, TextStyle, Ui, Vec2, Widget, WidgetInfo,
    WidgetText, WidgetType,
};
use egui_phosphor::regular;

mod top_tabs;

pub use top_tabs::{paint_top_tab_strip_underline, TopUnderlineTab};

pub use crate::layout::{TILE_GAP, TWO_LINE_TILE_HEIGHT};

pub fn padded_text_edit<'a>(text_edit: TextEdit<'a>) -> FocusTextEdit<'a> {
    FocusTextEdit {
        text_edit: text_edit.margin(layout::control_margin()).frame(false),
        round_right: true,
    }
}

pub fn edit_pulldown(
    ui: &mut Ui,
    id_source: impl std::hash::Hash + Copy,
    text: &mut String,
    hint: impl Into<WidgetText>,
    history: &[String],
    width: f32,
) -> Response {
    let popup_id = ui.make_persistent_id(("edit-pulldown", id_source));
    let selection_id = ui.make_persistent_id(("edit-pulldown-selection", id_source));
    let mut changed_by_selection = false;
    let response = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let arrow_width = square_icon_button_side() + layout::CONTROL_PADDING as f32;
            let text_width = (width - arrow_width).max(0.0);
            let text_response = ui.add_sized(
                [text_width, layout::CONTROL_HEIGHT],
                joined_pulldown_text_edit(TextEdit::singleline(text).hint_text(hint)),
            );
            if text_response.has_focus() || text_response.gained_focus() {
                ui.memory_mut(|mem| mem.open_popup(popup_id));
            }
            let arrow_response = ui
                .scope(|ui| {
                    ui.spacing_mut().button_padding.x = layout::CONTROL_PADDING as f32;
                    {
                        let widgets = &mut ui.visuals_mut().widgets;
                        for corner_radius in [
                            &mut widgets.inactive.corner_radius,
                            &mut widgets.hovered.corner_radius,
                            &mut widgets.active.corner_radius,
                            &mut widgets.open.corner_radius,
                        ] {
                            corner_radius.nw = 0;
                            corner_radius.sw = 0;
                        }
                        widgets.inactive.bg_stroke = egui::Stroke::NONE;
                        widgets.hovered.bg_stroke = egui::Stroke::NONE;
                        widgets.active.bg_stroke = egui::Stroke::NONE;
                        widgets.open.bg_stroke = egui::Stroke::NONE;
                    }
                    ui.add_sized(
                        [arrow_width, layout::CONTROL_HEIGHT],
                        egui::Button::new(egui_phosphor::regular::CARET_DOWN),
                    )
                })
                .inner;
            if arrow_response.clicked() {
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }
            let combined = text_response.union(arrow_response);
            let filter = text.to_ascii_lowercase();
            let filtered_items: Vec<&String> = history
                .iter()
                .filter(|item| filter.is_empty() || item.to_ascii_lowercase().contains(&filter))
                .collect();
            if text_response.has_focus() && !filtered_items.is_empty() {
                let down = ui.input(|input| input.key_pressed(egui::Key::ArrowDown));
                let up = ui.input(|input| input.key_pressed(egui::Key::ArrowUp));
                if down || up {
                    ui.memory_mut(|mem| mem.open_popup(popup_id));
                    let next_index = ui.ctx().data_mut(|data| {
                        let current = *data.get_temp_mut_or(selection_id, usize::MAX);
                        let next = if down {
                            current.saturating_add(1).min(filtered_items.len() - 1)
                        } else {
                            current.saturating_sub(1)
                        };
                        data.insert_temp(selection_id, next);
                        next
                    });
                    *text = filtered_items[next_index].clone();
                    changed_by_selection = true;
                }
            }
            egui::popup::popup_below_widget(
                ui,
                popup_id,
                &combined,
                egui::popup::PopupCloseBehavior::CloseOnClickOutside,
                |ui| {
                    ui.set_min_width(combined.rect.width());
                    for (index, item) in filtered_items.iter().enumerate() {
                        let selected = ui
                            .ctx()
                            .data_mut(|data| *data.get_temp_mut_or(selection_id, usize::MAX))
                            == index;
                        if ui.selectable_label(selected, *item).clicked() {
                            *text = (*item).clone();
                            ui.ctx()
                                .data_mut(|data| data.insert_temp(selection_id, index));
                            changed_by_selection = true;
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                    }
                },
            );
            combined
        })
        .inner;
    if changed_by_selection {
        let mut response = response;
        response.mark_changed();
        response
    } else {
        response
    }
}

fn joined_pulldown_text_edit<'a>(text_edit: TextEdit<'a>) -> FocusTextEdit<'a> {
    FocusTextEdit {
        text_edit: text_edit.margin(layout::control_margin()).frame(false),
        round_right: false,
    }
}

pub struct FocusTextEdit<'a> {
    text_edit: TextEdit<'a>,
    round_right: bool,
}

impl<'a> FocusTextEdit<'a> {
    pub fn hint_text(mut self, hint_text: impl Into<WidgetText>) -> Self {
        self.text_edit = self.text_edit.hint_text(hint_text);
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.text_edit = self.text_edit.password(password);
        self
    }

    pub fn font(mut self, text_style: TextStyle) -> Self {
        self.text_edit = self.text_edit.font(text_style);
        self
    }

    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.text_edit = self.text_edit.desired_width(desired_width);
        self
    }

    pub fn extra_right_margin(mut self, extra: i8) -> Self {
        let mut margin = layout::control_margin();
        margin.right = margin.right.saturating_add(extra);
        self.text_edit = self.text_edit.margin(margin);
        self
    }

    pub fn desired_rows(mut self, rows: usize) -> Self {
        self.text_edit = self.text_edit.desired_rows(rows);
        self
    }

    pub fn layouter(mut self, layouter: &'a mut dyn FnMut(&Ui, &str, f32) -> Arc<Galley>) -> Self {
        self.text_edit = self.text_edit.layouter(layouter);
        self
    }
}

impl Widget for FocusTextEdit<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let corner_radius = self.corner_radius(ui);
        let background = ui.painter().add(egui::Shape::Noop);
        let response = self.text_edit.ui(ui);
        let rect = response.rect.expand2(layout::control_padding());
        ui.painter().set(
            background,
            egui::Shape::rect_filled(rect, corner_radius, ui.visuals().extreme_bg_color),
        );
        if response.has_focus() {
            let bar = egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(
                    rect.left() + layout::TEXT_EDIT_FOCUS_BAR_WIDTH,
                    rect.bottom(),
                ),
            );
            ui.painter().rect_filled(
                bar,
                ui.visuals().widgets.inactive.corner_radius,
                ui.visuals().widgets.hovered.bg_fill,
            );
        }
        response
    }
}

impl FocusTextEdit<'_> {
    fn corner_radius(&self, ui: &Ui) -> egui::CornerRadius {
        let mut radius = ui.visuals().widgets.inactive.corner_radius;
        if !self.round_right {
            radius.ne = 0;
            radius.se = 0;
        }
        radius
    }
}

pub fn with_icon_button_padding<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = layout::control_padding();
        ui.spacing_mut().interact_size = Vec2::splat(square_icon_button_side());
        add_contents(ui)
    })
    .inner
}

pub fn square_icon_button_side() -> f32 {
    layout::square_icon_button_side()
}

pub fn square_icon_button_size() -> [f32; 2] {
    layout::square_icon_button_size()
}

pub fn checkbox(ui: &mut Ui, checked: &mut bool, text: impl Into<WidgetText>) -> Response {
    ui.add(IconCheckbox::new(checked, text))
}

pub fn checkbox_icon(checked: bool) -> &'static str {
    if checked {
        regular::CHECK_SQUARE
    } else {
        regular::SQUARE
    }
}

pub fn disable_tile_text_selection(ui: &mut Ui) {
    ui.style_mut().interaction.selectable_labels = false;
    ui.style_mut().interaction.multi_widget_text_select = false;
}

pub fn tile_list_content_width(ui: &Ui) -> f32 {
    ui.available_width().max(0.0)
}

pub fn tile_scroll_bar_rect(ui: &Ui) -> egui::Rect {
    let mut rect = ui.available_rect_before_wrap();
    rect.max.x = (rect.max.x - layout::TILE_SCROLLBAR_INSET).max(rect.min.x);
    rect
}

pub fn tile_scroll_bar_rect_with_height(ui: &Ui, height: f32) -> egui::Rect {
    let mut rect = tile_scroll_bar_rect(ui);
    rect.max.y = rect.max.y.min(rect.min.y + height.max(0.0));
    rect
}

pub fn fill_remaining_tile_rows(
    ui: &mut Ui,
    row_count: usize,
    row_height: f32,
    list_height: f32,
    tokens: ThemeTokens,
) {
    let used_height = row_count as f32 * row_height.max(0.0);
    let mut remaining = (list_height - used_height - 1.0).max(0.0);
    let mut index = row_count;
    while remaining > 0.0 {
        let height = remaining.min(row_height.max(1.0));
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::ZERO,
            tile_table_fill(index, tokens),
        );
        remaining -= height;
        index += 1;
    }
}

pub fn tile_inner_padding() -> egui::Vec2 {
    layout::tile_inner_padding()
}

pub fn tighten_tile_spacing(ui: &mut Ui) {
    ui.spacing_mut().item_spacing.y = layout::TILE_LINE_GAP;
}

pub fn tile_table_fill(index: usize, tokens: ThemeTokens) -> egui::Color32 {
    if index % 2 == 0 {
        tokens.panel_bg
    } else {
        tokens.panel_raised
    }
}

pub fn tile_table_hover_fill(tokens: ThemeTokens) -> egui::Color32 {
    highlight_fill(tokens)
}

pub fn tile_table_selected_fill(tokens: ThemeTokens) -> egui::Color32 {
    let selected = tokens.accent_selected_bg;
    if color_luminance(selected) < 128.0 {
        selected.gamma_multiply(0.82)
    } else {
        selected_light_blue(selected)
    }
}

pub fn highlight_fill(tokens: ThemeTokens) -> egui::Color32 {
    let highlight = tokens.accent_selected_bg;
    if color_luminance(highlight) < 128.0 {
        highlight
    } else {
        darker_light_blue(highlight)
    }
}

fn color_luminance(color: egui::Color32) -> f32 {
    0.2126 * color.r() as f32 + 0.7152 * color.g() as f32 + 0.0722 * color.b() as f32
}

fn darker_light_blue(color: egui::Color32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color.r() as f32 * 0.78).round() as u8,
        (color.g() as f32 * 0.88).round() as u8,
        color.b(),
        color.a(),
    )
}

fn selected_light_blue(color: egui::Color32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color.r() as f32 * 0.9).round() as u8,
        (color.g() as f32 * 0.96).round() as u8,
        color.b(),
        color.a(),
    )
}

pub fn tile_table_interactive_fill(
    index: usize,
    tokens: ThemeTokens,
    hovered: bool,
    selected: bool,
) -> egui::Color32 {
    if hovered {
        tile_table_hover_fill(tokens)
    } else if selected {
        tile_table_selected_fill(tokens)
    } else {
        tile_table_fill(index, tokens)
    }
}

pub fn dotted_focus_outline(ui: &Ui, rect: egui::Rect) {
    let color = if ui.visuals().dark_mode {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    };
    let stroke = egui::Stroke::new(1.0, color);
    let rect = rect.shrink(1.0);
    dotted_line(ui, rect.left_top(), rect.right_top(), stroke);
    dotted_line(ui, rect.right_top(), rect.right_bottom(), stroke);
    dotted_line(ui, rect.right_bottom(), rect.left_bottom(), stroke);
    dotted_line(ui, rect.left_bottom(), rect.left_top(), stroke);
}

fn dotted_line(ui: &Ui, start: egui::Pos2, end: egui::Pos2, stroke: egui::Stroke) {
    let delta = end - start;
    let length = delta.length();
    if length <= 0.0 {
        return;
    }
    let direction = delta / length;
    let dot = 2.0;
    let gap = 2.0;
    let mut offset = 0.0;
    while offset < length {
        let next = (offset + dot).min(length);
        ui.painter().line_segment(
            [start + direction * offset, start + direction * next],
            stroke,
        );
        offset += dot + gap;
    }
}

fn checkbox_icon_font(mut font: FontId) -> FontId {
    font.size *= layout::CHECKBOX_ICON_SCALE;
    font
}

struct IconCheckbox<'a> {
    checked: &'a mut bool,
    text: WidgetText,
}

impl<'a> IconCheckbox<'a> {
    fn new(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            text: text.into(),
        }
    }
}

impl Widget for IconCheckbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { checked, text } = self;
        let spacing = ui.spacing();
        let icon_side = spacing.interact_size.y.max(spacing.icon_width);
        let icon_spacing = spacing.icon_spacing;
        let has_text = !text.is_empty();
        let trailing_padding = if has_text {
            layout::CHECKBOX_TEXT_TRAILING_PADDING
        } else {
            0.0
        };

        let icon_font = checkbox_icon_font(TextStyle::Button.resolve(ui.style()));
        let icon_galley = ui.painter().layout_no_wrap(
            checkbox_icon(*checked).to_owned(),
            icon_font,
            egui::Color32::PLACEHOLDER,
        );
        let galley = if has_text {
            let wrap_width =
                (ui.available_width() - icon_side - icon_spacing - trailing_padding).max(0.0);
            Some(text.into_galley(ui, None, wrap_width, TextStyle::Button))
        } else {
            None
        };

        let text_size = galley.as_ref().map_or(Vec2::ZERO, |galley| galley.size());
        let desired_size = if has_text {
            vec2(
                (icon_side + icon_spacing + text_size.x + trailing_padding)
                    .max(spacing.interact_size.x),
                icon_side.max(text_size.y).max(spacing.interact_size.y),
            )
        } else {
            Vec2::splat(icon_side.max(spacing.interact_size.y))
        };

        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        if response.clicked() {
            *checked = !*checked;
            response.mark_changed();
        }
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::Checkbox,
                ui.is_enabled(),
                *checked,
                galley.as_ref().map_or("", |galley| galley.text()),
            )
        });

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            if response.hovered() || response.has_focus() || response.is_pointer_button_down_on() {
                ui.painter().rect(
                    rect,
                    visuals.corner_radius,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            let icon_color = ui.visuals().weak_text_color();
            let icon_pos = pos2(
                rect.left() + (icon_side - icon_galley.size().x) * 0.5,
                rect.center().y - icon_galley.size().y * 0.5,
            );
            ui.painter().galley(icon_pos, icon_galley, icon_color);

            if let Some(galley) = galley {
                let text_pos = pos2(
                    rect.left() + icon_side + icon_spacing,
                    rect.center().y - galley.size().y * 0.5,
                );
                ui.painter().galley(text_pos, galley, visuals.text_color());
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{static_tokens, ThemeSelection};

    #[test]
    fn checkbox_icons_match_styling_spec() {
        assert_eq!(checkbox_icon(false), regular::SQUARE);
        assert_eq!(checkbox_icon(true), regular::CHECK_SQUARE);
    }

    #[test]
    fn checkbox_icon_font_scales_button_font_size() {
        let font = FontId::proportional(13.0);
        assert_eq!(checkbox_icon_font(font).size, 20.800001);
    }

    #[test]
    fn tile_table_fills_are_distinct_from_each_other() {
        let tokens = static_tokens(&ThemeSelection::Dark);
        assert_ne!(tile_table_fill(0, tokens), tile_table_fill(1, tokens));
    }

    #[test]
    fn icon_button_size_is_square_control_height() {
        let [width, height] = square_icon_button_size();
        assert_eq!(width, height);
        assert_eq!(width, layout::CONTROL_HEIGHT);
    }
}
