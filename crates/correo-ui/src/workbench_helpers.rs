use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionState, QosLevel};
use egui::{Align2, Button, ComboBox, CornerRadius, FontId, Frame, Rect, RichText, Ui, UiBuilder};

use crate::theme::{ThemeTokens, CONTROL_HEIGHT};
use crate::widgets::paint_focus_outline;

pub(crate) fn connected(snapshot: &AppSnapshot) -> bool {
    snapshot
        .selected_connection()
        .is_some_and(|connection| connection.state == ConnectionState::Connected)
}

pub(crate) fn disconnected_action_button(
    ui: &mut Ui,
    width: f32,
    label: impl Into<String>,
    tooltip: &'static str,
    tokens: ThemeTokens,
) {
    let label = label.into();
    let response = ui
        .add_enabled_ui(false, |ui| {
            ui.spacing_mut().button_padding.x = 4.0;
            ui.add_sized([width, CONTROL_HEIGHT], Button::new(&label))
        })
        .inner;
    paint_focus_outline(ui, &response);
    let hover_response = ui.interact(
        response.rect,
        response.id.with("disabled-action-warning"),
        egui::Sense::hover(),
    );

    if hover_response.hovered() || hover_response.contains_pointer() {
        let text_color = contrast_text(tokens.danger);
        ui.painter().rect_filled(
            response.rect,
            CornerRadius::same(correo_style::layout::CORNER_RADIUS),
            tokens.danger,
        );
        ui.painter().text(
            response.rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(14.0),
            text_color,
        );
    }

    hover_response.on_hover_ui(|ui| {
        let text_color = contrast_text(tokens.danger);
        Frame::NONE
            .fill(tokens.danger)
            .corner_radius(CornerRadius::same(correo_style::layout::CORNER_RADIUS))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.label(RichText::new(tooltip).strong().color(text_color));
            });
    });
}

pub(crate) fn qos_selector(
    ui: &mut Ui,
    id: &'static str,
    current: QosLevel,
    mut on_change: impl FnMut(QosLevel),
) {
    let mut selected = current;
    let response = ComboBox::from_id_salt(id)
        .selected_text(current.label())
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for qos in QosLevel::ALL {
                ui.selectable_value(&mut selected, qos, qos.label());
            }
        })
        .response;
    paint_focus_outline(ui, &response);
    if selected != current {
        on_change(selected);
    }
}

pub(crate) fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}

pub(crate) fn toolbar_rect(ui: &mut Ui) -> Rect {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), crate::theme::CONTROL_HEIGHT),
        egui::Sense::hover(),
    );
    rect
}

fn contrast_text(background: egui::Color32) -> egui::Color32 {
    let [red, green, blue, _] = background.to_array();
    let luminance = 0.299 * red as f32 + 0.587 * green as f32 + 0.114 * blue as f32;
    if luminance > 140.0 {
        egui::Color32::from_rgb(0x17, 0x20, 0x2A)
    } else {
        egui::Color32::WHITE
    }
}

pub(crate) fn right_rect(row: Rect, width: f32, right_offset: f32) -> Rect {
    Rect::from_min_max(
        egui::pos2(row.right() - right_offset - width, row.top()),
        egui::pos2(row.right() - right_offset, row.bottom()),
    )
}

pub(crate) fn child_ui(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) {
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.set_clip_rect(rect.expand(2.0));
    add(&mut child);
}
