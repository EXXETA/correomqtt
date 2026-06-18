use correo_core::{AppCommand, AppCommandSender, WorkbenchTab};
use correo_style::layout;
use egui::{
    pos2, vec2, Align, Button, Color32, CursorIcon, Id, Layout, Rect, RichText, Sense, Stroke, Ui,
    UiBuilder,
};
use egui_phosphor::regular;

use crate::{
    responsive,
    theme::ThemeTokens,
    widgets::{square_icon_button_size, with_icon_button_padding},
};

const PANE_TITLE_SIZE: f32 = 18.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchPaneSide {
    Publish,
    Subscribe,
}

pub(crate) fn show(
    ui: &mut Ui,
    tokens: ThemeTokens,
    active_tab: WorkbenchTab,
    commands: &AppCommandSender,
    publish: impl FnOnce(&mut Ui),
    subscribe: impl FnOnce(&mut Ui),
    outgoing: impl FnOnce(&mut Ui),
    incoming: impl FnOnce(&mut Ui),
) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    let natural_tabs = rect.width() < layout::WORKBENCH_NARROW_WIDTH;
    let forced_tabs = forced_tab_mode(ui);
    let tabs_visible = natural_tabs || forced_tabs;
    responsive::set_workbench_tabs_visible(ui.ctx(), tabs_visible);
    if tabs_visible {
        tabbed_layout(
            ui,
            rect,
            tokens,
            active_tab,
            commands,
            natural_tabs,
            publish,
            subscribe,
            outgoing,
            incoming,
        );
        return;
    }

    let (left, right, divider) = center_split(ui, rect, tokens);
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
    divider_mode_button(ui, divider);
}

pub(crate) fn pane_title(ui: &mut Ui, title: &str, _side: WorkbenchPaneSide) {
    if responsive::workbench_tabs_visible(ui) {
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
    commands: &AppCommandSender,
    natural_tabs: bool,
    publish: impl FnOnce(&mut Ui),
    subscribe: impl FnOnce(&mut Ui),
    outgoing: impl FnOnce(&mut Ui),
    incoming: impl FnOnce(&mut Ui),
) {
    let tab_height = layout::CONTROL_HEIGHT + 8.0;
    let tab_bottom = (rect.top() + tab_height).min(rect.bottom());
    let tab_rect = Rect::from_min_max(rect.left_top(), pos2(rect.right(), tab_bottom));
    tab_bar(ui, tab_rect, active_tab, commands, natural_tabs, tokens);
    let content = Rect::from_min_max(pos2(rect.left(), tab_bottom), rect.right_bottom());
    match active_tab {
        WorkbenchTab::Publish => stack_split(
            ui,
            Id::new("workbench-narrow-publish-stack-ratio"),
            content,
            tokens,
            publish,
            outgoing,
        ),
        WorkbenchTab::Subscribe => stack_split(
            ui,
            Id::new("workbench-narrow-subscribe-stack-ratio"),
            content,
            tokens,
            subscribe,
            incoming,
        ),
    }
}

fn tab_bar(
    ui: &mut Ui,
    rect: Rect,
    active_tab: WorkbenchTab,
    commands: &AppCommandSender,
    natural_tabs: bool,
    _tokens: ThemeTokens,
) {
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.set_clip_rect(rect);
    child.spacing_mut().item_spacing.x = layout::TOOLBAR_GAP;
    let mode_button_width = square_icon_button_size()[0];
    let tab_width =
        ((rect.width() - mode_button_width - (layout::TOOLBAR_GAP * 2.0)) * 0.5).max(0.0);
    for tab in [WorkbenchTab::Publish, WorkbenchTab::Subscribe] {
        let selected = active_tab == tab;
        let label = if selected {
            let color = if child.visuals().dark_mode {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            RichText::new(tab.label()).color(color)
        } else {
            RichText::new(tab.label())
        };
        let response = child.allocate_ui_with_layout(
            egui::vec2(tab_width, layout::CONTROL_HEIGHT),
            Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| ui.selectable_label(selected, label),
        );
        if response.inner.clicked() {
            let _ = commands.send(AppCommand::SelectWorkbenchTab(tab));
        }
    }
    tab_mode_button(&mut child, natural_tabs);
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
    draw_divider(ui, divider, tokens.border, true);

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

fn divider_mode_button(ui: &mut Ui, divider: Rect) {
    let size = egui::Vec2::from(square_icon_button_size());
    let rect = Rect::from_center_size(
        pos2(
            divider.center().x,
            divider.top() - (size.y * 0.5) - layout::TOOLBAR_GAP,
        ),
        size,
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
    );
    child.set_clip_rect(rect.expand(2.0));
    if mode_button(&mut child, regular::TABS, true, "Use tab mode").clicked() {
        set_forced_tab_mode(&child, true);
    }
}

fn tab_mode_button(ui: &mut Ui, natural_tabs: bool) {
    let tooltip = if natural_tabs {
        "Split mode is unavailable at this width"
    } else {
        "Use split mode"
    };
    if mode_button(ui, regular::COLUMNS, !natural_tabs, tooltip).clicked() {
        set_forced_tab_mode(ui, false);
    }
}

fn mode_button(ui: &mut Ui, icon: &str, enabled: bool, tooltip: &str) -> egui::Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_enabled(
            enabled,
            Button::new(RichText::new(icon).size(16.0))
                .min_size(egui::Vec2::from(square_icon_button_size())),
        )
    })
    .on_hover_text(tooltip)
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
    draw_divider(ui, divider, tokens.border, false);

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

fn draw_divider(ui: &Ui, rect: Rect, color: Color32, vertical: bool) {
    let center = rect.center();
    let points = if vertical {
        [pos2(center.x, rect.top()), pos2(center.x, rect.bottom())]
    } else {
        [pos2(rect.left(), center.y), pos2(rect.right(), center.y)]
    };
    ui.painter().line_segment(points, Stroke::new(1.0, color));
}
