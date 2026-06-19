use crate::{layout, ThemeTokens};
use egui::{pos2, vec2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget, WidgetText};

const TAB_UNDERLINE_WIDTH: f32 = 2.0;

pub struct TopUnderlineTab {
    label: WidgetText,
    selected: bool,
    tokens: ThemeTokens,
    size: Vec2,
}

impl TopUnderlineTab {
    pub fn new(label: impl Into<WidgetText>, selected: bool, tokens: ThemeTokens) -> Self {
        Self {
            label: label.into(),
            selected,
            tokens,
            size: vec2(96.0, layout::CONTROL_HEIGHT),
        }
    }

    pub fn size(mut self, size: impl Into<Vec2>) -> Self {
        self.size = size.into();
        self
    }
}

impl Widget for TopUnderlineTab {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            label,
            selected,
            tokens,
            size,
        } = self;
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() || response.has_focus();
            if hovered || selected {
                let fill = if hovered {
                    tokens.accent
                } else {
                    selected_tab_fill(ui, tokens)
                };
                ui.painter().rect_filled(rect, top_corner_radius(), fill);
            }

            let text_color = if hovered {
                egui::Color32::WHITE
            } else if selected {
                tokens.accent
            } else {
                tokens.text_primary
            };
            let galley = label.into_galley(ui, None, rect.width(), egui::TextStyle::Button);
            let text_pos = pos2(
                rect.center().x - galley.size().x * 0.5,
                rect.center().y - galley.size().y * 0.5,
            );
            ui.painter().galley(text_pos, galley, text_color);
        }

        response
    }
}

pub fn paint_top_tab_strip_underline(
    ui: &Ui,
    strip_rect: Rect,
    active_rect: Rect,
    tokens: ThemeTokens,
) {
    let y = strip_rect.bottom() - (TAB_UNDERLINE_WIDTH * 0.5);
    ui.painter().line_segment(
        [pos2(strip_rect.left(), y), pos2(strip_rect.right(), y)],
        Stroke::new(TAB_UNDERLINE_WIDTH, tokens.border),
    );
    ui.painter().line_segment(
        [pos2(active_rect.left(), y), pos2(active_rect.right(), y)],
        Stroke::new(TAB_UNDERLINE_WIDTH, tokens.accent),
    );
}

fn top_corner_radius() -> egui::CornerRadius {
    egui::CornerRadius {
        nw: layout::CORNER_RADIUS,
        ne: layout::CORNER_RADIUS,
        sw: 0,
        se: 0,
    }
}

fn selected_tab_fill(ui: &Ui, tokens: ThemeTokens) -> egui::Color32 {
    if ui.visuals().dark_mode {
        tokens.panel_raised.gamma_multiply(1.2)
    } else {
        tokens.panel_raised.gamma_multiply(0.94)
    }
}
