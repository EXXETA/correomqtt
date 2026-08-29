use correo_core::{sample_snapshot, ThemeMode, Workspace};
use correo_ui::CorreoUi;
use egui::accesskit::Role;
use egui_kittest::{kittest::Queryable as _, Harness};

#[test]
fn message_toolbar_icon_buttons_have_accessible_names() {
    let mut snapshot = sample_snapshot(ThemeMode::Light);
    snapshot.active_workspace = Workspace::Connections;
    let mut shell = CorreoUi::for_snapshot(snapshot);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build(move |context| shell.draw(context));

    harness.run();

    harness.get_by_role_and_label(Role::Button, "Copy message to publish form");
    harness.get_by_role_and_label(Role::Button, "Show message in separate window");
    harness.get_by_role_and_label(Role::Button, "Clear messages");
    harness.get_by_role_and_label(Role::Button, "Toggle automatic scrolling");
}
