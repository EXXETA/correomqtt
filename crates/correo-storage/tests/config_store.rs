use correo_storage::current::{
    AppConfig, Auth, BuiltInBrokerConfig, ConfigStore, ConnectionConfig, Lwt, MqttVersion, Proxy,
    Settings, Theme, ThemeSettings, TlsSsl,
};

fn connection(id: &str) -> ConnectionConfig {
    ConnectionConfig {
        id: id.to_owned(),
        name: "Synthetic Broker".to_owned(),
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

    let mut settings = Settings::default();
    settings.saved_locale = Some("de_DE".to_owned());
    settings.current_locale = Some("en_US".to_owned());
    settings.use_regex_for_search = true;
    settings.use_ignore_case = true;
    settings.search_updates = true;
    settings.keyring_identifier = Some("KWallet5".to_owned());
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
