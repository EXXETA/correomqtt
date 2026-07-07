use correo_storage::current::{
    AppConfig, Auth, BuiltInBrokerConfig, ConfigStore, ConnectionConfig, ConnectionUiSettings, Lwt,
    MqttVersion, Protocol, Proxy, Settings, Theme, ThemeSettings, TlsSsl,
};

fn connection(id: &str) -> ConnectionConfig {
    ConnectionConfig {
        id: id.to_owned(),
        name: "Synthetic Broker".to_owned(),
        protocol: Protocol::Mqtt,
        url: "localhost".to_owned(),
        port: 1883,
        client_id: Some("correo-test".to_owned()),
        username: Some("synthetic-user".to_owned()),
        clean_session: true,
        mqtt_version: MqttVersion::Mqtt311,
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
fn saves_global_settings_without_replacing_connections() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());
    let mut config = AppConfig {
        connections: vec![connection("connection-01")],
        theme_settings: Some(ThemeSettings {
            active_theme: Some(Theme {
                name: Some("Light".to_owned()),
            }),
        }),
        settings: Settings::default(),
        built_in_broker: BuiltInBrokerConfig::default(),
    };
    config.settings.keyring_identifier = Some("LibSecret".to_owned());
    store.save(&config).unwrap();

    let mut settings = Settings {
        saved_locale: Some("de_DE".to_owned()),
        current_locale: Some("en_US".to_owned()),
        use_regex_for_search: true,
        use_ignore_case: true,
        search_updates: true,
        keyring_identifier: Some("KWallet5".to_owned()),
        ..Default::default()
    };
    settings.plugin_repositories.insert(
        "synthetic".to_owned(),
        "https://example.invalid/plugins.json".to_owned(),
    );

    let saved = store.save_global_settings("Dark", settings).unwrap();
    assert_eq!(saved.connections.len(), 1);

    let loaded = store.load().unwrap();
    assert_eq!(loaded.connections[0].id, "connection-01");
    assert_eq!(
        loaded.theme_settings.unwrap().active_theme.unwrap().name,
        Some("Dark".to_owned())
    );
    assert_eq!(loaded.settings.saved_locale.as_deref(), Some("de_DE"));
    assert!(loaded.settings.use_regex_for_search);
    assert!(loaded.settings.use_ignore_case);
    assert_eq!(
        loaded.settings.keyring_identifier.as_deref(),
        Some("KWallet5")
    );
    assert_eq!(
        loaded.settings.plugin_repositories.get("synthetic"),
        Some(&"https://example.invalid/plugins.json".to_owned())
    );
}

#[test]
fn connection_without_protocol_field_defaults_to_mqtt() {
    // Simulates a profile written before the protocol discriminator existed:
    // the field is absent on disk and must load as MQTT (serde default).
    let mut value = serde_json::to_value(connection("legacy-broker")).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .remove("protocol")
        .expect("fresh configs serialize the protocol field");
    assert!(value.get("protocol").is_none());

    let restored: ConnectionConfig = serde_json::from_value(value).unwrap();
    assert_eq!(restored.protocol, Protocol::Mqtt);
}

#[test]
fn saved_connection_round_trips_protocol() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());
    store.save_connection(connection("proto-broker")).unwrap();

    let loaded = store.load().unwrap();
    assert_eq!(loaded.connections[0].protocol, Protocol::Mqtt);
}

#[test]
fn form_save_preserves_migrated_ui_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());

    // A connection carrying UI metadata (as migration or the workbench would
    // persist it).
    let mut with_ui = connection("ui-broker");
    with_ui.connection_ui_settings = Some(ConnectionUiSettings {
        show_publish: true,
        main_divider_position: 0.42,
        ..ConnectionUiSettings::default()
    });
    store.save_connection(with_ui).unwrap();

    // The settings form re-saves the same connection with these form-external
    // fields as None; they must survive.
    let mut form_save = connection("ui-broker");
    form_save.name = "Renamed".to_owned();
    form_save.connection_ui_settings = None;
    store.save_connection(form_save).unwrap();

    let loaded = store.load().unwrap();
    let connection = &loaded.connections[0];
    assert_eq!(connection.name, "Renamed");
    let ui = connection
        .connection_ui_settings
        .as_ref()
        .expect("migrated UI metadata must not be wiped by a form save");
    assert!(ui.show_publish);
    assert_eq!(ui.main_divider_position, 0.42);
}
