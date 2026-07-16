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

#[test]
fn broker_password_persistence_writes_only_a_secret_reference() {
    use correo_storage::current::SecretStore;

    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());
    let secret_store = MemorySecretStore::default();
    let password = "broker-password-in-keyring";

    let command = SettingsPersistenceCommand::SaveBuiltInBroker {
        broker: crate::BuiltInBrokerPersistenceSnapshot {
            port: "1883".to_owned(),
            credentials_enabled: true,
            username: "broker".to_owned(),
            password: password.to_owned(),
        },
    };
    assert!(!format!("{command:?}").contains(password));

    super::save_built_in_broker(
        &store,
        &secret_store,
        crate::BuiltInBrokerPersistenceSnapshot {
            port: "1883".to_owned(),
            credentials_enabled: true,
            username: "broker".to_owned(),
            password: password.to_owned(),
        },
    )
    .unwrap();

    let persisted = std::fs::read_to_string(temp.path().join("config.json")).unwrap();
    assert!(!persisted.contains(password));
    let config: serde_json::Value = serde_json::from_str(&persisted).unwrap();
    assert!(config.pointer("/built_in_broker/password").is_none());
    let reference = store
        .load_built_in_broker_secret_reference()
        .unwrap()
        .expect("broker reference is persisted privately");
    assert_eq!(
        reference.kind,
        correo_storage::current::SecretKind::Password
    );
    assert!(reference.connection_id.starts_with("builtin-broker-"));
    let secret = secret_store
        .get(&reference)
        .unwrap()
        .expect("persisted broker password must be stored in the secret store");
    assert_eq!(secret.expose_secret(), password);
}

#[test]
fn legacy_broker_password_migrates_to_stable_secret_reference() {
    use correo_storage::current::SecretStore;

    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());
    let legacy_password = "legacy-broker-password";
    std::fs::write(
        temp.path().join("config.json"),
        format!(
            r#"{{"built_in_broker":{{"port":"1883","credentials_enabled":true,"username":"broker","password":"{legacy_password}"}}}}"#
        ),
    )
    .unwrap();
    let secret_store = MemorySecretStore::default();

    let migrated = super::migrate_legacy_built_in_broker(&store, &secret_store).unwrap();

    let reference = store
        .load_built_in_broker_secret_reference()
        .unwrap()
        .expect("migrated broker reference is persisted privately");
    assert_eq!(
        reference.kind,
        correo_storage::current::SecretKind::Password
    );
    assert!(reference.connection_id.starts_with("builtin-broker-"));
    assert!(migrated.built_in_broker.password.is_empty());
    assert!(!std::fs::read_to_string(temp.path().join("config.json"))
        .unwrap()
        .contains(legacy_password));
    assert_eq!(
        secret_store
            .get(&reference)
            .unwrap()
            .map(|secret| secret.expose_secret().to_owned()),
        Some(legacy_password.to_owned())
    );
}

#[test]
fn failed_broker_secret_write_retains_legacy_config() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());
    let legacy_password = "legacy-broker-password";
    std::fs::write(
        temp.path().join("config.json"),
        format!(
            r#"{{"built_in_broker":{{"port":"1883","credentials_enabled":true,"username":"broker","password":"{legacy_password}"}}}}"#
        ),
    )
    .unwrap();

    let result = super::save_built_in_broker(
        &store,
        &FailingSecretStore,
        crate::BuiltInBrokerPersistenceSnapshot {
            port: "1883".to_owned(),
            credentials_enabled: true,
            username: "broker".to_owned(),
            password: legacy_password.to_owned(),
        },
    );

    assert!(result.is_err());
    assert!(
        std::fs::read_to_string(temp.path().join("config.json"))
            .unwrap()
            .contains(legacy_password),
        "the legacy config must remain intact when writing its replacement secret fails"
    );
}

#[test]
fn connection_and_broker_passwords_coexist_for_the_built_in_broker_id() {
    use correo_storage::current::{SecretKind, SecretMaterial, SecretReference, SecretStore};

    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());
    let secret_store = MemorySecretStore::default();
    let mut settings = connection_settings("Reserved", "localhost");
    settings.password = crate::SecretInput::new("connection-password");

    super::save_connection_settings(
        &store,
        &secret_store,
        "built-in-broker".to_owned(),
        settings,
    )
    .unwrap();
    super::save_built_in_broker(
        &store,
        &secret_store,
        crate::BuiltInBrokerPersistenceSnapshot {
            port: "1883".to_owned(),
            credentials_enabled: true,
            username: "broker".to_owned(),
            password: "broker-password".to_owned(),
        },
    )
    .unwrap();

    let connection_reference = SecretReference {
        connection_id: "built-in-broker".to_owned(),
        kind: SecretKind::Password,
    };
    let broker_reference = store
        .load_built_in_broker_secret_reference()
        .unwrap()
        .expect("broker reference is persisted privately");
    assert_eq!(
        secret_store
            .get(&connection_reference)
            .unwrap()
            .map(SecretMaterial::expose_for_migration),
        Some("connection-password".to_owned())
    );
    assert_eq!(
        secret_store
            .get(&broker_reference)
            .unwrap()
            .map(SecretMaterial::expose_for_migration),
        Some("broker-password".to_owned())
    );
}
