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
        .with_size(egui::vec2(1280.0, 800.0))
        .build(move |context| shell.draw(context));

    harness.run();

    for label in [
        "Copy message to publish form",
        "Show message in separate window",
        "Clear messages",
        "Toggle automatic scrolling",
    ] {
        assert!(
            harness
                .query_all_by_role_and_label(Role::Button, label)
                .next()
                .is_some(),
            "missing button: {label}"
        );
    }
}
