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
