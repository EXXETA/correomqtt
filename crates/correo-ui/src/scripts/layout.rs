use egui::{pos2, vec2, Align, CursorIcon, Id, Layout, Rect, Sense, Ui, UiBuilder};

use crate::theme::ThemeTokens;
use crate::widgets::paint_pane_divider;

const DEFAULT_UPPER_RATIO: f32 = 0.65;
const DIVIDER_SIZE: f32 = 8.0;
const MIN_UPPER_HEIGHT: f32 = 220.0;
const MIN_LOWER_HEIGHT: f32 = 160.0;
const PANE_PADDING_X: f32 = 10.0;
const PANE_PADDING_TOP: f32 = 10.0;
const PANE_PADDING_BOTTOM: f32 = 10.0;

pub(super) fn right_panes(
    ui: &mut Ui,
    tokens: ThemeTokens,
    top: impl FnOnce(&mut Ui),
    bottom: impl FnOnce(&mut Ui),
) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    let (top_rect, bottom_rect) = vertical_split(ui, Id::new("scripts-upper-ratio"), rect, tokens);
    pane(ui, top_rect, 0.0, top);
    pane(ui, bottom_rect, PANE_PADDING_TOP, bottom);
}

pub(super) fn list_column(
    ui: &mut Ui,
    tokens: ThemeTokens,
    top: impl FnOnce(&mut Ui),
    bottom: impl FnOnce(&mut Ui),
) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    let (top_rect, bottom_rect) =
        vertical_split(ui, Id::new("scripts-list-column-ratio"), rect, tokens);
    pane(ui, top_rect, 0.0, top);
    pane(ui, bottom_rect, PANE_PADDING_TOP, bottom);
}

fn vertical_split(ui: &mut Ui, id: Id, rect: Rect, tokens: ThemeTokens) -> (Rect, Rect) {
    let usable = (rect.height() - DIVIDER_SIZE).max(1.0);
    let min_top = MIN_UPPER_HEIGHT.min(usable * 0.55);
    let min_bottom = MIN_LOWER_HEIGHT.min((usable - min_top).max(0.0));
    let max_top = (usable - min_bottom).max(min_top);
    let mut top = ratio(ui, id, DEFAULT_UPPER_RATIO) * usable;
    top = top.clamp(min_top, max_top);
    let divider = Rect::from_min_size(
        pos2(rect.left(), rect.top() + top),
        vec2(rect.width(), DIVIDER_SIZE),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeVertical);
    if response.dragged() {
        top = (top + response.drag_delta().y).clamp(min_top, max_top);
        store_ratio(ui, id, top / usable);
    }
    paint_pane_divider(ui, divider, &response, tokens, false);
    (
        Rect::from_min_max(rect.left_top(), pos2(rect.right(), divider.top())),
        Rect::from_min_max(pos2(rect.left(), divider.bottom()), rect.right_bottom()),
    )
}

fn pane(ui: &mut Ui, rect: Rect, top_padding: f32, contents: impl FnOnce(&mut Ui)) {
    let content_rect = Rect::from_min_max(
        pos2(rect.left() + PANE_PADDING_X, rect.top() + top_padding),
        pos2(
            rect.right() - PANE_PADDING_X,
            rect.bottom() - PANE_PADDING_BOTTOM,
        ),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(content_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(content_rect);
    contents(&mut child);
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
