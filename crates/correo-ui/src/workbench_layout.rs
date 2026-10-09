use correo_core::{AppCommand, AppCommandSender, WorkbenchTab};
use correo_style::layout;
use egui::{pos2, vec2, Align, CursorIcon, Id, Layout, Rect, RichText, Sense, Ui, UiBuilder};

use crate::{
    responsive,
    theme::ThemeTokens,
    widgets::{
        dotted_focus_outline, paint_pane_divider, paint_top_tab_strip_underline, TopUnderlineTab,
    },
};

const PANE_TITLE_SIZE: f32 = 18.0;
const TITLE_TAB_GAP: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchPaneSide {
    Publish,
    Subscribe,
}

pub(crate) fn show(
    ui: &mut Ui,
    tokens: ThemeTokens,
    active_tab: WorkbenchTab,
    publish: impl FnOnce(&mut Ui),
    subscribe: impl FnOnce(&mut Ui),
    outgoing: impl FnOnce(&mut Ui),
    incoming: impl FnOnce(&mut Ui),
) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    let tabs_visible = tabs_visible_for_width(ui, rect.width());
    responsive::set_workbench_tabs_visible(ui.ctx(), tabs_visible);
    if tabs_visible {
        tabbed_layout(
            ui, rect, tokens, active_tab, publish, subscribe, outgoing, incoming,
        );
        return;
    }

    let (left, right, _) = center_split(ui, rect, tokens);
    stack_split(
        ui,
        Id::new("workbench-left-stack-ratio"),
        left,
        tokens,
        publish,
        outgoing,
    );
    stack_split(
        ui,
        Id::new("workbench-right-stack-ratio"),
        right,
        tokens,
        subscribe,
        incoming,
    );
}

pub(crate) fn pane_title(
    ui: &mut Ui,
    title: &str,
    _side: WorkbenchPaneSide,
    active_tab: WorkbenchTab,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if responsive::workbench_tabs_visible(ui) {
        pane_tab_controls(ui, tokens, active_tab, commands);
        return;
    }

    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), layout::CONTROL_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(RichText::new(title).strong().size(PANE_TITLE_SIZE));
        },
    );
}

fn tabbed_layout(
    ui: &mut Ui,
    rect: Rect,
    tokens: ThemeTokens,
    active_tab: WorkbenchTab,
    publish: impl FnOnce(&mut Ui),
    subscribe: impl FnOnce(&mut Ui),
    outgoing: impl FnOnce(&mut Ui),
    incoming: impl FnOnce(&mut Ui),
) {
    match active_tab {
        WorkbenchTab::Publish => stack_split(
            ui,
            Id::new("workbench-narrow-publish-stack-ratio"),
            rect,
            tokens,
            publish,
            outgoing,
        ),
        WorkbenchTab::Subscribe => stack_split(
            ui,
            Id::new("workbench-narrow-subscribe-stack-ratio"),
            rect,
            tokens,
            subscribe,
            incoming,
        ),
    }
}

fn pane_tab_controls(
    ui: &mut Ui,
    tokens: ThemeTokens,
    active_tab: WorkbenchTab,
    commands: &AppCommandSender,
) {
    let rect = Rect::from_min_size(
        ui.available_rect_before_wrap().min,
        vec2(ui.available_width(), layout::CONTROL_HEIGHT),
    );
    ui.allocate_rect(rect, Sense::hover());
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.set_clip_rect(rect);
    child.spacing_mut().item_spacing.x = 0.0;
    let tab_width = ((rect.width() - TITLE_TAB_GAP) * 0.5).max(0.0);
    let mut active_rect = None;
    let mut focused_rect = None;
    for (index, tab) in [WorkbenchTab::Publish, WorkbenchTab::Subscribe]
        .into_iter()
        .enumerate()
    {
        let selected = active_tab == tab;
        let tab_label = if selected {
            RichText::new(tab.label()).strong()
        } else {
            RichText::new(tab.label())
        };
        let response = child.add(
            TopUnderlineTab::new(tab_label, selected, tokens)
                .size(vec2(tab_width, layout::CONTROL_HEIGHT)),
        );
        if selected {
            active_rect = Some(response.rect);
        }
        if response.has_focus() {
            focused_rect = Some(response.rect);
        }
        if response.clicked() {
            let _ = commands.send(AppCommand::SelectWorkbenchTab(tab));
        }
        if index == 0 {
            child.add_space(TITLE_TAB_GAP);
        }
    }
    if let Some(active_rect) = active_rect {
        let strip_rect = Rect::from_min_size(
            rect.min,
            vec2(tab_width * 2.0 + TITLE_TAB_GAP, layout::CONTROL_HEIGHT),
        );
        paint_top_tab_strip_underline(&child, strip_rect, active_rect, tokens);
    }
    if let Some(focused_rect) = focused_rect {
        dotted_focus_outline(&child, focused_rect);
    }
}

pub(crate) fn natural_tabs_for_width(width: f32) -> bool {
    width < layout::WORKBENCH_NARROW_WIDTH
}

pub(crate) fn tabs_visible_for_width(ui: &Ui, width: f32) -> bool {
    natural_tabs_for_width(width) || forced_tab_mode(ui)
}

pub(crate) fn tab_toggle_available(width: f32) -> bool {
    !natural_tabs_for_width(width)
}

pub(crate) fn user_tab_mode(ui: &Ui) -> bool {
    forced_tab_mode(ui)
}

pub(crate) fn toggle_user_tab_mode(ui: &Ui) {
    set_forced_tab_mode(ui, !forced_tab_mode(ui));
}

fn center_split(ui: &mut Ui, rect: Rect, tokens: ThemeTokens) -> (Rect, Rect, Rect) {
    let usable = (rect.width() - layout::WORKBENCH_DIVIDER_SIZE).max(1.0);
    let min_left = layout::WORKBENCH_MIN_PANE_WIDTH.min(usable * 0.45);
    let min_right = layout::WORKBENCH_MIN_PANE_WIDTH.min((usable - min_left).max(0.0));
    let max_left = (usable - min_right).max(min_left);
    let id = Id::new("workbench-center-ratio");
    let mut left_width = ratio(ui, id, layout::WORKBENCH_DEFAULT_CENTER_RATIO) * usable;
    left_width = left_width.clamp(min_left, max_left);

    let divider = Rect::from_min_size(
        pos2(rect.left() + left_width, rect.top()),
        vec2(layout::WORKBENCH_DIVIDER_SIZE, rect.height()),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeHorizontal);
    if response.dragged() {
        left_width = (left_width + response.drag_delta().x).clamp(min_left, max_left);
        store_ratio(ui, id, left_width / usable);
    }
    paint_pane_divider(ui, divider, &response, tokens, true);

    let left = Rect::from_min_max(
        rect.left_top(),
        pos2(
            divider.left() - layout::WORKBENCH_CENTER_SPLIT_GUTTER,
            rect.bottom(),
        ),
    );
    let right = Rect::from_min_max(
        pos2(
            divider.right() + layout::WORKBENCH_CENTER_SPLIT_GUTTER,
            rect.top(),
        ),
        rect.right_bottom(),
    );
    (left, right, divider)
}

fn stack_split(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    tokens: ThemeTokens,
    top: impl FnOnce(&mut Ui),
    bottom: impl FnOnce(&mut Ui),
) {
    let usable = (rect.height() - layout::WORKBENCH_DIVIDER_SIZE).max(1.0);
    let min_top = layout::WORKBENCH_MIN_TOP_HEIGHT.min(usable * 0.6);
    let min_bottom = layout::WORKBENCH_MIN_BOTTOM_HEIGHT.min((usable - min_top).max(0.0));
    let max_top = (usable - min_bottom).max(min_top);
    let mut top_height = ratio(ui, id, layout::WORKBENCH_DEFAULT_STACK_RATIO) * usable;
    top_height = top_height.clamp(min_top, max_top);

    let divider = Rect::from_min_size(
        pos2(rect.left(), rect.top() + top_height),
        vec2(rect.width(), layout::WORKBENCH_DIVIDER_SIZE),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeVertical);
    if response.dragged() {
        top_height = (top_height + response.drag_delta().y).clamp(min_top, max_top);
        store_ratio(ui, id, top_height / usable);
    }
    paint_pane_divider(ui, divider, &response, tokens, false);

    top_pane(
        ui,
        Rect::from_min_max(rect.left_top(), pos2(rect.right(), divider.top())),
        top,
    );
    bottom_pane(
        ui,
        Rect::from_min_max(pos2(rect.left(), divider.bottom()), rect.right_bottom()),
        bottom,
    );
}

fn top_pane(ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut Ui)) {
    pane(ui, rect, layout::WORKBENCH_PANE_PADDING_Y, add_contents);
}

fn bottom_pane(ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut Ui)) {
    pane(ui, rect, 0.0, add_contents);
}

fn pane(ui: &mut Ui, rect: Rect, bottom_padding: f32, add_contents: impl FnOnce(&mut Ui)) {
    let content_rect = Rect::from_min_max(
        pos2(
            rect.left() + layout::WORKBENCH_PANE_PADDING_X,
            rect.top() + layout::WORKBENCH_PANE_PADDING_Y,
        ),
        pos2(
            rect.right() - layout::WORKBENCH_PANE_PADDING_X,
            rect.bottom() - bottom_padding,
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

fn forced_tab_mode(ui: &Ui) -> bool {
    ui.ctx()
        .data_mut(|data| data.get_persisted(forced_tab_mode_id()).unwrap_or(false))
}

fn set_forced_tab_mode(ui: &Ui, enabled: bool) {
    ui.ctx()
        .data_mut(|data| data.insert_persisted(forced_tab_mode_id(), enabled));
}

fn forced_tab_mode_id() -> Id {
    Id::new("workbench-force-tab-mode")
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
