use std::time::Duration;

use correo_storage::current::{
    Auth, ConfigStore, ConnectionConfig, Lwt, MqttVersion, Protocol, Proxy, TlsSsl,
};

use crate::{
    available_keyring_options, ConnectionSettingsSnapshot, GlobalSettingFlag,
    GlobalSettingsSnapshot, PluginRepositoryRow, SettingsPersistenceCommand,
    SettingsPersistenceEvent, SettingsPersistenceWorker, ThemeMode,
};

fn start_worker(path: &std::path::Path) -> SettingsPersistenceWorker {
    static MOCK_KEYRING: std::sync::Once = std::sync::Once::new();
    MOCK_KEYRING.call_once(|| {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    });
    SettingsPersistenceWorker::start(path)
}

#[test]
fn worker_persists_global_settings_off_the_caller_thread() {
    let temp = tempfile::tempdir().unwrap();
    let worker = start_worker(temp.path());
    let mut settings = GlobalSettingsSnapshot::default();
    let keyring_backend = available_keyring_options()
        .into_iter()
        .find(|option| option.id != "os")
        .expect("at least one explicit keyring backend should be available")
        .id;
    settings.language = "de_DE".to_owned();
    settings.search_use_regex = true;
    settings.search_ignore_case = true;
    settings.reduce_motion = true;
    settings.keyring_backend = keyring_backend.clone();
    settings.plugin_repositories = vec![
        PluginRepositoryRow {
            id: "empty".to_owned(),
            url: "  ".to_owned(),
        },
        PluginRepositoryRow {
            id: "custom".to_owned(),
            url: "https://example.invalid/plugins.json".to_owned(),
        },
    ];

    worker
        .dispatch(SettingsPersistenceCommand::Save {
            theme_mode: ThemeMode::Dark,
            settings: Box::new(settings),
        })
        .unwrap();

    assert_eq!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(SettingsPersistenceEvent::Saved)
    );

    let config = ConfigStore::new(temp.path()).load().unwrap();
    assert_eq!(config.settings.saved_locale.as_deref(), Some("de_DE"));
    assert!(config.settings.use_regex_for_search);
    assert!(config.settings.use_ignore_case);
    assert!(config.settings.reduce_motion);
    assert_eq!(
        config.settings.keyring_identifier.as_deref(),
        Some(keyring_backend.as_str())
    );
    assert_eq!(config.settings.plugin_repositories.len(), 1);
    assert_eq!(
        config.settings.plugin_repositories.get("custom"),
        Some(&"https://example.invalid/plugins.json".to_owned())
    );
    assert_eq!(
        config
            .theme_settings
            .unwrap()
            .active_theme
            .unwrap()
            .name
            .as_deref(),
        Some(correo_style::DARK_THEME_ID)
    );
}

#[test]
fn settings_flag_enum_stays_exhaustive_for_persistence() {
    let _ = GlobalSettingFlag::InstallBundledPlugins;
    let _ = GlobalSettingFlag::ReduceMotion;
}

#[test]
fn worker_persists_new_connection_without_keyring_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let worker = start_worker(temp.path());
    let settings = connection_settings("Local broker", "localhost");

    worker
        .dispatch(SettingsPersistenceCommand::SaveConnectionSettings {
            connection_id: "new-connection-01".to_owned(),
            settings: Box::new(settings),
        })
        .unwrap();

    assert_eq!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(SettingsPersistenceEvent::Saved)
    );

    let config = ConfigStore::new(temp.path()).load().unwrap();
    assert_eq!(config.connections.len(), 1);
    assert_eq!(config.connections[0].id, "new-connection-01");
    assert_eq!(config.connections[0].name, "Local broker");
    assert_eq!(config.connections[0].url, "localhost");
}

#[test]
fn worker_deletes_connection_from_config() {
    let temp = tempfile::tempdir().unwrap();
    let worker = start_worker(temp.path());
    save_connection(&worker, "connection-01", "First", "first.local");
    save_connection(&worker, "connection-02", "Second", "second.local");

    worker
        .dispatch(SettingsPersistenceCommand::DeleteConnection {
            connection_id: "connection-01".to_owned(),
        })
        .unwrap();

    assert_eq!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(SettingsPersistenceEvent::Saved)
    );

    let config = ConfigStore::new(temp.path()).load().unwrap();
    assert_eq!(config.connections.len(), 1);
    assert_eq!(config.connections[0].id, "connection-02");
}

#[test]
fn worker_persists_connection_order() {
    let temp = tempfile::tempdir().unwrap();
    let worker = start_worker(temp.path());
    save_connection(&worker, "connection-01", "First", "first.local");
    save_connection(&worker, "connection-02", "Second", "second.local");

    worker
        .dispatch(SettingsPersistenceCommand::SaveConnectionOrder {
            connection_ids: vec!["connection-02".to_owned(), "connection-01".to_owned()],
        })
        .unwrap();

    assert_eq!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(SettingsPersistenceEvent::Saved)
    );

    let config = ConfigStore::new(temp.path()).load().unwrap();
    let ids = config
        .connections
        .iter()
        .map(|connection| connection.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["connection-02", "connection-01"]);
}

#[test]
fn worker_persists_imported_connections_without_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let worker = start_worker(temp.path());

    worker
        .dispatch(SettingsPersistenceCommand::SaveImportedConnections {
            connections: vec![connection_config(
                "imported-01",
                "Imported",
                "imported.local",
            )],
            secrets: Vec::new(),
        })
        .unwrap();

    assert_eq!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(SettingsPersistenceEvent::Saved)
    );

    let config = ConfigStore::new(temp.path()).load().unwrap();
    assert_eq!(config.connections.len(), 1);
    assert_eq!(config.connections[0].id, "imported-01");
    assert_eq!(config.connections[0].name, "Imported");
    assert_eq!(config.connections[0].url, "imported.local");
}

fn save_connection(
    worker: &SettingsPersistenceWorker,
    connection_id: &str,
    name: &str,
    host: &str,
) {
    worker
        .dispatch(SettingsPersistenceCommand::SaveConnectionSettings {
            connection_id: connection_id.to_owned(),
            settings: Box::new(connection_settings(name, host)),
        })
        .unwrap();
    assert_eq!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(SettingsPersistenceEvent::Saved)
    );
}

fn connection_settings(name: &str, host: &str) -> ConnectionSettingsSnapshot {
    ConnectionSettingsSnapshot {
        profile_name: name.to_owned(),
        host: host.to_owned(),
        port: "1883".to_owned(),
        mqtt_version: "MQTT 5.0".to_owned(),
        clean_session: true,
        password_status: "No MQTT password configured".to_owned(),
        tls_mode: "No TLS/SSL".to_owned(),
        tls_password_status: "No SSL password configured".to_owned(),
        tls_host_verification: true,
        proxy_mode: "No Proxy".to_owned(),
        ssh_port: "22".to_owned(),
        auth_mode: "No Auth".to_owned(),
        ssh_password_status: "No SSH password configured".to_owned(),
        valid: true,
        ..ConnectionSettingsSnapshot::default()
    }
}

fn connection_config(id: &str, name: &str, host: &str) -> ConnectionConfig {
    ConnectionConfig {
        id: id.to_owned(),
        name: name.to_owned(),
        protocol: Protocol::Mqtt,
        url: host.to_owned(),
        port: 1883,
        client_id: None,
        username: None,
        clean_session: true,
        mqtt_version: MqttVersion::Mqtt50,
        ssl: TlsSsl::Off,
        ssl_keystore: None,
        ssl_host_verification: true,
        proxy: Proxy::Off,
        ssh_host: None,
        ssh_port: 22,
        local_port: None,
        auth: Auth::Off,
        auth_username: None,
        auth_keyfile: None,
        lwt: Lwt::Off,
        lwt_topic: None,
        lwt_qos: None,
        lwt_retained: false,
        lwt_payload: None,
        connection_ui_settings: None,
        publish_list_view_config: None,
        subscribe_list_view_config: None,
        plugin_workflows: Vec::new(),
    }
}

struct FailingSecretStore;

impl correo_storage::current::SecretStore for FailingSecretStore {
    fn put(
        &self,
        _reference: &correo_storage::current::SecretReference,
        _value: &correo_storage::current::SecretMaterial,
    ) -> correo_storage::Result<()> {
        Err(correo_storage::StorageError::SecretStore {
            operation: "write",
            reference: "injected".to_owned(),
            message: "secret store unavailable".to_owned(),
        })
    }

    fn get(
        &self,
        _reference: &correo_storage::current::SecretReference,
    ) -> correo_storage::Result<Option<correo_storage::current::SecretMaterial>> {
        Ok(None)
    }

    fn delete(
        &self,
        _reference: &correo_storage::current::SecretReference,
    ) -> correo_storage::Result<()> {
        Ok(())
    }
}

#[test]
fn failed_secret_write_does_not_persist_connection() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());
    let mut settings = connection_settings("Broker", "localhost");
    settings.password = crate::SecretInput::new("hunter2");

    let result = super::save_connection_settings(
        &store,
        &FailingSecretStore,
        "connection-01".to_owned(),
        settings,
    );

    assert!(result.is_err(), "secret failure must surface as an error");
    // Secrets are written before the config, so a failed secret write leaves
    // no config file (or an empty one) — never a connection without credentials.
    let persisted = store
        .load()
        .map(|config| config.connections.len())
        .unwrap_or(0);
    assert_eq!(
        persisted, 0,
        "connection must not be persisted when its secret write failed"
    );
}

#[test]
fn failed_config_write_rolls_back_secret_changes() {
    use correo_storage::current::{
        ImportedSecret, OsKeyringSecretStore, SecretKind, SecretMaterial, SecretReference,
        SecretStore,
    };

    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    let secret_store = OsKeyringSecretStore::new("test.rollback.config");
    let reference = SecretReference {
        connection_id: "c1".to_owned(),
        kind: SecretKind::Password,
    };
    secret_store
        .put(&reference, &SecretMaterial::new("old-secret"))
        .unwrap();

    let new_put = ImportedSecret {
        reference: reference.clone(),
        value: SecretMaterial::new("new-secret"),
    };
    let result =
        super::apply_secrets_then_save(&secret_store, std::slice::from_ref(&new_put), &[], || {
            Err(correo_storage::StorageError::SecretStore {
                operation: "write",
                reference: "config".to_owned(),
                message: "injected config failure".to_owned(),
            })
        });

    assert!(result.is_err(), "config failure must surface as an error");
    let restored = secret_store.get(&reference).unwrap();
    assert_eq!(
        restored.map(SecretMaterial::expose_for_migration),
        Some("old-secret".to_owned()),
        "secret must roll back to its previous value when the config write fails"
    );
}
