pub(crate) use correo_style::widgets::*;

use egui::{Align2, Color32, CursorIcon, FontId, Id, Response, Sense, TextEdit, Ui, WidgetText};
use egui_phosphor::regular;

const SEARCH_CLEAR_ICON_SIZE: f32 = 13.0;
const MENU_ITEM_HEIGHT: f32 = 30.0;
const MENU_ITEM_PADDING_X: f32 = 12.0;
const MENU_ITEM_ICON_WIDTH: f32 = 16.0;
const MENU_ITEM_ICON_GAP: f32 = 9.0;
const MENU_ITEM_ICON_SIZE: f32 = 14.0;
const MENU_ITEM_WIDTH_ID: &str = "correo-menu-item-width";
const MENU_ITEM_ICON_COLUMN_ID: &str = "correo-menu-item-icon-column";

pub(crate) fn clearable_search_edit(
    ui: &mut Ui,
    id: Option<Id>,
    text: &mut String,
    hint: impl Into<WidgetText>,
    width: f32,
) -> Response {
    let mut edit = TextEdit::singleline(text).hint_text(hint);
    if let Some(id) = id {
        edit = edit.id(id);
    }
    let mut response = ui.add_sized(
        [width, crate::theme::CONTROL_HEIGHT],
        padded_text_edit(edit),
    );

    if !text.is_empty() {
        let control_rect = response
            .rect
            .expand2(correo_style::layout::control_padding());
        let side = crate::theme::CONTROL_HEIGHT;
        let clear_rect = egui::Rect::from_center_size(
            egui::pos2(control_rect.right() - side * 0.5, control_rect.center().y),
            egui::vec2(side, side),
        );
        let clear_response = ui
            .interact(clear_rect, response.id.with("clear"), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        if clear_response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            ui.painter().rect_filled(
                clear_rect.shrink(5.0),
                ui.visuals().widgets.hovered.corner_radius,
                ui.visuals().widgets.hovered.bg_fill,
            );
        }
        let color = if clear_response.hovered() {
            ui.visuals().widgets.hovered.fg_stroke.color
        } else {
            ui.visuals().widgets.inactive.fg_stroke.color
        };
        ui.painter().text(
            clear_rect.center(),
            Align2::CENTER_CENTER,
            regular::X,
            FontId::proportional(SEARCH_CLEAR_ICON_SIZE),
            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 210),
        );
        if clear_response.clicked() {
            text.clear();
            response.mark_changed();
        }
    }

    response
}

pub(crate) fn menu_item(ui: &mut Ui, icon: Option<&str>, label: &str) -> Response {
    menu_item_enabled(ui, true, icon, label)
}

pub(crate) fn menu_item_enabled(
    ui: &mut Ui,
    enabled: bool,
    icon: Option<&str>,
    label: &str,
) -> Response {
    let text_font = FontId::proportional(ui.text_style_height(&egui::TextStyle::Button) * 0.82);
    let content_width = menu_item_width_with_font(ui, label, text_font.clone());
    let width = configured_menu_item_width(ui).max(content_width);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, MENU_ITEM_HEIGHT), sense);
    let response = if enabled {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response
    };

    let visuals = ui.style().interact_selectable(&response, false);
    if enabled && (response.hovered() || response.has_focus()) {
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, visuals.bg_fill);
    }

    let color = if enabled {
        visuals.fg_stroke.color
    } else {
        ui.visuals().weak_text_color()
    };
    let icon_column = configured_menu_item_icon_column(ui);
    let text_left = if icon_column {
        rect.left() + MENU_ITEM_PADDING_X + MENU_ITEM_ICON_WIDTH + MENU_ITEM_ICON_GAP
    } else {
        rect.left() + MENU_ITEM_PADDING_X
    };
    if let (true, Some(icon)) = (icon_column, icon) {
        let icon_left = rect.left() + MENU_ITEM_PADDING_X;
        ui.painter().text(
            egui::pos2(icon_left + MENU_ITEM_ICON_WIDTH * 0.5, rect.center().y),
            Align2::CENTER_CENTER,
            icon,
            FontId::proportional(MENU_ITEM_ICON_SIZE),
            color,
        );
    }
    ui.painter().text(
        egui::pos2(text_left, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        text_font,
        color,
    );

    response
}

pub(crate) fn set_menu_item_width(ui: &mut Ui, labels: &[&str]) {
    set_menu_item_width_with_icons(ui, labels, true);
}

pub(crate) fn set_text_menu_item_width(ui: &mut Ui, labels: &[&str]) {
    set_menu_item_width_with_icons(ui, labels, false);
}

fn set_menu_item_width_with_icons(ui: &mut Ui, labels: &[&str], icon_column: bool) {
    let width = menu_item_content_width_with_icons(ui, labels, icon_column);
    ui.set_min_width(width);
    ui.ctx().data_mut(|data| {
        data.insert_temp(menu_item_width_id(), width);
        data.insert_temp(menu_item_icon_column_id(), icon_column);
    });
}

pub(crate) fn menu_item_content_width(ui: &Ui, labels: &[&str]) -> f32 {
    menu_item_content_width_with_icons(ui, labels, true)
}

pub(crate) fn menu_item_content_width_without_icons(ui: &Ui, labels: &[&str]) -> f32 {
    menu_item_content_width_with_icons(ui, labels, false)
}

fn menu_item_content_width_with_icons(ui: &Ui, labels: &[&str], icon_column: bool) -> f32 {
    labels.iter().fold(0.0_f32, |width, label| {
        width.max(menu_item_width(ui, label, icon_column))
    })
}

fn menu_item_width(ui: &Ui, label: &str, icon_column: bool) -> f32 {
    let font = FontId::proportional(ui.text_style_height(&egui::TextStyle::Button) * 0.82);
    menu_item_width_with_font_and_icons(ui, label, font, icon_column)
}

fn menu_item_width_with_font(ui: &Ui, label: &str, font: FontId) -> f32 {
    menu_item_width_with_font_and_icons(ui, label, font, configured_menu_item_icon_column(ui))
}

fn menu_item_width_with_font_and_icons(
    ui: &Ui,
    label: &str,
    font: FontId,
    icon_column: bool,
) -> f32 {
    let label_width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, ui.visuals().text_color())
        .size()
        .x;
    let icon_width = if icon_column {
        MENU_ITEM_ICON_WIDTH + MENU_ITEM_ICON_GAP
    } else {
        0.0
    };
    MENU_ITEM_PADDING_X * 2.0 + icon_width + label_width
}

fn configured_menu_item_width(ui: &Ui) -> f32 {
    ui.ctx()
        .data_mut(|data| data.get_temp::<f32>(menu_item_width_id()))
        .unwrap_or(0.0)
}

fn menu_item_width_id() -> Id {
    Id::new(MENU_ITEM_WIDTH_ID)
}

fn configured_menu_item_icon_column(ui: &Ui) -> bool {
    ui.ctx()
        .data_mut(|data| data.get_temp::<bool>(menu_item_icon_column_id()))
        .unwrap_or(true)
}

fn menu_item_icon_column_id() -> Id {
    Id::new(MENU_ITEM_ICON_COLUMN_ID)
}
