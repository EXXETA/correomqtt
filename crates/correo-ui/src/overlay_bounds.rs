const CONNECTION_OVERLAY_RECT_ID: &str = "connection-overlay-rect";

pub(crate) fn set(ui: &egui::Ui, rect: egui::Rect) {
    ui.ctx().data_mut(|data| {
        data.insert_temp(
            id(),
            rect.expand(f32::from(correo_style::layout::CENTRAL_MARGIN)),
        )
    });
}

pub(crate) fn get(ui: &egui::Ui) -> egui::Rect {
    ui.ctx()
        .data_mut(|data| data.get_temp::<egui::Rect>(id()))
        .unwrap_or_else(|| ui.max_rect())
}

fn id() -> egui::Id {
    egui::Id::new(CONNECTION_OVERLAY_RECT_ID)
}
