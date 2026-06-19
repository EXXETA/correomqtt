use egui::{
    pos2, vec2, Align, Color32, CursorIcon, Id, Layout, Rect, Sense, Stroke, Ui, UiBuilder,
};

use crate::theme::ThemeTokens;

const DEFAULT_UPPER_RATIO: f32 = 0.65;
const DEFAULT_LEFT_RATIO: f32 = 0.35;
const DIVIDER_SIZE: f32 = 8.0;
const MIN_PANE_WIDTH: f32 = correo_style::layout::SCRIPTING_FLYOUT_WIDTH;
const MIN_RIGHT_PANE_WIDTH: f32 = 550.0;
const MIN_UPPER_HEIGHT: f32 = 220.0;
const MIN_LOWER_HEIGHT: f32 = 160.0;
const PANE_PADDING_X: f32 = 10.0;
const PANE_PADDING_TOP: f32 = 10.0;
const PANE_PADDING_BOTTOM: f32 = 10.0;

pub(super) fn four_pane(
    ui: &mut Ui,
    tokens: ThemeTokens,
    top_left: impl FnOnce(&mut Ui),
    top_right: impl FnOnce(&mut Ui),
    bottom_left: impl FnOnce(&mut Ui),
    bottom_right: impl FnOnce(&mut Ui),
    divider_control: impl FnOnce(&mut Ui, Rect),
) {
    let full_rect = ui.available_rect_before_wrap();
    ui.allocate_rect(full_rect, Sense::hover());

    let (top_rect, bottom_rect) =
        vertical_split(ui, Id::new("scripts-upper-ratio"), full_rect, tokens);
    horizontal_split_with_control(
        ui,
        shared_list_ratio_id(),
        top_rect,
        tokens,
        0.0,
        top_left,
        top_right,
        Some(divider_control),
    );
    horizontal_split_with_control(
        ui,
        shared_list_ratio_id(),
        bottom_rect,
        tokens,
        PANE_PADDING_TOP,
        bottom_left,
        bottom_right,
        None::<fn(&mut Ui, Rect)>,
    );
}

pub(super) fn right_panes(
    ui: &mut Ui,
    tokens: ThemeTokens,
    top_right: impl FnOnce(&mut Ui),
    bottom_right: impl FnOnce(&mut Ui),
) {
    let full_rect = ui.available_rect_before_wrap();
    ui.allocate_rect(full_rect, Sense::hover());
    let (top_rect, bottom_rect) =
        vertical_split(ui, Id::new("scripts-upper-ratio"), full_rect, tokens);
    pane(ui, top_rect, 0.0, top_right);
    pane(ui, bottom_rect, PANE_PADDING_TOP, bottom_right);
}

pub(super) fn list_column(
    ui: &mut Ui,
    tokens: ThemeTokens,
    top_left: impl FnOnce(&mut Ui),
    bottom_left: impl FnOnce(&mut Ui),
) {
    let full_rect = ui.available_rect_before_wrap();
    ui.allocate_rect(full_rect, Sense::hover());
    let (top_rect, bottom_rect) = vertical_split(
        ui,
        Id::new("scripts-flyout-list-column-ratio"),
        full_rect,
        tokens,
    );
    pane_with_padding(ui, top_rect, 0.0, 0.0, top_left);
    pane_with_padding(ui, bottom_rect, 0.0, PANE_PADDING_TOP, bottom_left);
}

fn vertical_split(ui: &mut Ui, id: Id, rect: Rect, tokens: ThemeTokens) -> (Rect, Rect) {
    let usable = (rect.height() - DIVIDER_SIZE).max(1.0);
    let min_upper = MIN_UPPER_HEIGHT.min(usable * 0.55);
    let min_lower = MIN_LOWER_HEIGHT.min((usable - min_upper).max(0.0));
    let max_upper = (usable - min_lower).max(min_upper);
    let mut upper = ratio(ui, id, DEFAULT_UPPER_RATIO) * usable;
    upper = upper.clamp(min_upper, max_upper);

    let divider = Rect::from_min_size(
        pos2(rect.left(), rect.top() + upper),
        vec2(rect.width(), DIVIDER_SIZE),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeVertical);
    if response.dragged() {
        upper = (upper + response.drag_delta().y).clamp(min_upper, max_upper);
        store_ratio(ui, id, upper / usable);
    }
    draw_divider(ui, divider, tokens.border, false);

    let top = Rect::from_min_max(rect.left_top(), pos2(rect.right(), divider.top()));
    let bottom = Rect::from_min_max(pos2(rect.left(), divider.bottom()), rect.right_bottom());
    (top, bottom)
}

fn horizontal_split_with_control(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    tokens: ThemeTokens,
    top_padding: f32,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
    divider_control: Option<impl FnOnce(&mut Ui, Rect)>,
) {
    let usable = (rect.width() - DIVIDER_SIZE).max(1.0);
    let min_left = MIN_PANE_WIDTH.min(usable * 0.45);
    let min_right = MIN_RIGHT_PANE_WIDTH.min((usable - min_left).max(0.0));
    let max_left = (usable - min_right).max(min_left);
    let mut left_width = ratio(ui, id, DEFAULT_LEFT_RATIO) * usable;
    left_width = left_width.clamp(min_left, max_left);

    let divider = Rect::from_min_size(
        pos2(rect.left() + left_width, rect.top()),
        vec2(DIVIDER_SIZE, rect.height()),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeHorizontal);
    if response.dragged() {
        left_width = (left_width + response.drag_delta().x).clamp(min_left, max_left);
        store_ratio(ui, id, left_width / usable);
    }
    draw_divider(ui, divider, tokens.border, true);
    if let Some(divider_control) = divider_control {
        divider_control(ui, divider);
    }

    pane(
        ui,
        Rect::from_min_max(
            rect.left_top(),
            pos2(divider.left() - PANE_PADDING_X, rect.bottom()),
        ),
        top_padding,
        left,
    );
    pane(
        ui,
        Rect::from_min_max(
            pos2(divider.right() + PANE_PADDING_X, rect.top()),
            rect.right_bottom(),
        ),
        top_padding,
        right,
    );
}

fn shared_list_ratio_id() -> Id {
    Id::new("scripts-list-ratio")
}

fn pane(ui: &mut Ui, rect: Rect, top_padding: f32, add_contents: impl FnOnce(&mut Ui)) {
    pane_with_padding(ui, rect, PANE_PADDING_X, top_padding, add_contents);
}

fn pane_with_padding(
    ui: &mut Ui,
    rect: Rect,
    horizontal_padding: f32,
    top_padding: f32,
    add_contents: impl FnOnce(&mut Ui),
) {
    let content_rect = Rect::from_min_max(
        pos2(rect.left() + horizontal_padding, rect.top() + top_padding),
        pos2(
            rect.right() - horizontal_padding,
            rect.bottom() - PANE_PADDING_BOTTOM,
        ),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(content_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(content_rect);
    add_contents(&mut child);
}

fn ratio(ui: &Ui, id: Id, default: f32) -> f32 {
    ui.ctx()
        .data_mut(|data| *data.get_persisted_mut_or(id, default))
        .clamp(0.15, 0.85)
}

fn store_ratio(ui: &Ui, id: Id, value: f32) {
    ui.ctx()
        .data_mut(|data| data.insert_persisted(id, value.clamp(0.15, 0.85)));
}

fn draw_divider(ui: &Ui, rect: Rect, color: Color32, vertical: bool) {
    let center = rect.center();
    let points = if vertical {
        [
            pos2(center.x, rect.top()),
            pos2(center.x, rect.bottom() - PANE_PADDING_BOTTOM),
        ]
    } else {
        [pos2(rect.left(), center.y), pos2(rect.right(), center.y)]
    };
    ui.painter().line_segment(points, Stroke::new(1.0, color));
}
