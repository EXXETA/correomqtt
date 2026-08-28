use std::path::{Path, PathBuf};

use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::MigrationPreview;

use super::AppModel;
use crate::{
    startup_state_from_migration, AppCommand, ConnectionId, ConnectionSecretField, SecretInput,
    ThemeMode,
};

fn storage_fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../correo-storage/tests/fixtures")
        .join(path)
}

fn first_user_connection_id(model: &AppModel) -> ConnectionId {
    model
        .snapshot()
        .connections
        .iter()
        .find(|connection| !connection.immutable)
        .expect("user connection exists")
        .id
}

#[test]
fn migrated_connection_settings_expose_legacy_form_fields() {
    let profile = LegacyProfile::read_from(storage_fixture("legacy_profile")).unwrap();
    let preview = MigrationPreview::from_legacy_profile(profile).unwrap();
    let state = startup_state_from_migration(preview, ThemeMode::Dark);
    let mut model = AppModel::with_startup_state(state);
    let first_id = first_user_connection_id(&model);

    model.apply_command(AppCommand::OpenConnectionSettings(first_id));
    let settings = &model.snapshot().connection_settings;

    assert_eq!(settings.internal_id, "local-broker-01");
    assert_eq!(settings.client_id, "correo-rust-test");
    assert_eq!(settings.username, "synthetic-user");
    assert!(!settings.clean_session);
    assert_eq!(settings.tls_mode, "No TLS/SSL");
    assert!(!settings.tls_host_verification);
    assert_eq!(settings.proxy_mode, "SSH");
    assert_eq!(settings.ssh_host, "ssh.example.invalid");
    assert_eq!(settings.ssh_port, "22");
    assert_eq!(settings.local_mqtt_port, "11883");
    assert_eq!(settings.auth_mode, "Keyfile");
    assert_eq!(settings.auth_username, "synthetic-ssh-user");
    assert_eq!(settings.ssh_key_file, "/synthetic/path/id_ed25519");
    assert!(settings.lwt_retained);
    assert_eq!(settings.lwt_payload, "offline");
}

#[test]
fn connection_password_drafts_are_redacted_from_debug_surfaces() {
    let mut model = AppModel::empty();
    model.apply_command(AppCommand::AddConnection);
    model.apply_command(AppCommand::UpdateConnectionSecret {
        field: ConnectionSecretField::MqttPassword,
        value: SecretInput::new("synthetic-form-password"),
    });

    let snapshot_debug = format!("{:?}", model.snapshot());
    assert!(!snapshot_debug.contains("synthetic-form-password"));

    let command_debug = format!(
        "{:?}",
        AppCommand::UpdateConnectionSecret {
            field: ConnectionSecretField::MqttPassword,
            value: SecretInput::new("synthetic-command-password"),
        }
    );
    assert!(!command_debug.contains("synthetic-command-password"));
}
