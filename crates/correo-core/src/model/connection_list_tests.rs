use super::AppModel;
use crate::{AppCommand, ConnectionId, ConnectionSurface};

fn user_connection_ids(model: &AppModel) -> Vec<ConnectionId> {
    model
        .snapshot()
        .connections
        .iter()
        .filter(|connection| !connection.immutable)
        .map(|connection| connection.id)
        .collect()
}

#[test]
fn connection_selection_opens_workbench_and_preserves_pending_editor() {
    let mut model = AppModel::default();
    let user_ids = user_connection_ids(&model);
    let first_id = user_ids[0];
    let second_id = user_ids[1];

    model.apply_command(AppCommand::OpenConnectionSettings(first_id));

    assert_eq!(model.snapshot().selected_connection, Some(first_id));
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert_eq!(model.snapshot().connection_settings_overlay, Some(first_id));

    model.apply_command(AppCommand::SelectConnection(second_id));

    assert_eq!(model.snapshot().selected_connection, Some(second_id));
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert_eq!(model.snapshot().connection_settings_overlay, Some(first_id));

    model.apply_command(AppCommand::SelectConnection(first_id));
    assert_eq!(model.snapshot().connection_settings_overlay, Some(first_id));

    model.apply_command(AppCommand::DiscardConnectionSettings);
    assert_eq!(model.snapshot().connection_settings_overlay, None);
}

#[test]
fn launcher_command_selects_first_connection_workbench() {
    let mut model = AppModel::default();
    let first_id = model.snapshot().connections[0].id;

    model.apply_command(AppCommand::AddConnection);
    assert_eq!(model.snapshot().selected_connection, None);

    model.apply_command(AppCommand::OpenConnectionLauncher);

    assert_eq!(model.snapshot().selected_connection, Some(first_id));
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
}

#[test]
fn delete_connection_removes_it_and_selects_first_available() {
    let mut model = AppModel::default();
    let user_ids = user_connection_ids(&model);
    let second_id = user_ids[1];
    let original_count = model.snapshot().connection_count;

    let second_name = model
        .snapshot()
        .connections
        .iter()
        .find(|connection| connection.id == second_id)
        .expect("second user connection exists")
        .name
        .clone();

    model.apply_command(AppCommand::SelectConnection(second_id));
    model.apply_command(AppCommand::RequestDeleteConnection);
    assert!(
        model
            .snapshot()
            .connection_settings
            .delete_confirmation_open
    );
    assert_eq!(
        model.snapshot().connection_settings.profile_name,
        second_name
    );

    model.apply_command(AppCommand::ConfirmDeleteConnection);

    assert_eq!(model.snapshot().connection_count, original_count - 1);
    assert!(!model
        .snapshot()
        .connections
        .iter()
        .any(|connection| connection.id == second_id));
    assert_eq!(
        model.snapshot().selected_connection,
        Some(model.snapshot().connections[0].id)
    );
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert_eq!(model.snapshot().connection_settings_overlay, None);
    assert!(
        !model
            .snapshot()
            .connection_settings
            .delete_confirmation_open
    );
}

#[test]
fn move_connection_reorders_visible_connections() {
    let mut model = AppModel::default();
    let original = user_connection_ids(&model);

    model.apply_command(AppCommand::MoveConnection {
        connection_id: original[0],
        target_connection_id: original[2],
        after: true,
    });

    let reordered = user_connection_ids(&model);
    assert_eq!(
        reordered,
        [original[1], original[2], original[0], original[3]]
    );

    model.apply_command(AppCommand::MoveConnection {
        connection_id: original[0],
        target_connection_id: original[1],
        after: false,
    });

    let restored_front = user_connection_ids(&model);
    assert_eq!(restored_front, original);
}
