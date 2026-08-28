#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PluginLoadState, PluginSource, PluginStatus};

    const TEST_REPOSITORY_JSON: &str =
        include_str!("../../correo-plugins/tests/fixtures/repository.json");

    fn bundled_plugin_ids() -> Vec<String> {
        [
            "org.correomqtt.plugins.advanced-validator",
            "org.correomqtt.plugins.base64",
            "org.correomqtt.plugins.contains-string-validator",
            "org.correomqtt.plugins.json-format",
            "org.correomqtt.plugins.save-manipulator",
            "org.correomqtt.plugins.system-topic",
            "org.correomqtt.plugins.xml-format",
            "org.correomqtt.plugins.xml-xsd-validator",
            "org.correomqtt.plugins.zip-manipulator",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }

    #[test]
    fn current_startup_populates_marketplace_and_installs_bundled_plugins() {
        let state = startup_state_from_current_with_plugins(
            AppConfig::default(),
            HistoryPersistenceSnapshot::default(),
            BTreeMap::new(),
            ScriptPersistenceSnapshot::default(),
            Vec::new(),
            ThemeMode::System,
            vec![TEST_REPOSITORY_JSON.to_owned()],
            bundled_plugin_ids(),
            Vec::new(),
            Vec::new(),
        );
        let plugins = &state.snapshot.plugins;

        assert_eq!(plugins.load_state, PluginLoadState::Ready);
        assert_eq!(plugins.marketplace_plugins.len(), 9);
        assert_eq!(plugins.plugins.len(), 9);
        assert!(!plugins.selected_plugin_id.is_empty());
        assert!(!plugins.selected_marketplace_plugin_id.is_empty());
        assert!(plugins.plugins.iter().all(|plugin| {
            plugin.source == PluginSource::Bundled && plugin.status == PluginStatus::Active
        }));
        assert!(plugins
            .marketplace_plugins
            .iter()
            .filter(|plugin| plugin.install_source.is_bundled())
            .all(|plugin| plugin.installed_plugin_id.as_deref() == Some(plugin.id.as_str())));
        assert_eq!(
            plugins
                .marketplace_plugins
                .iter()
                .find(|plugin| plugin.id == "org.correomqtt.plugins.save-manipulator")
                .and_then(|plugin| plugin.installed_plugin_id.as_deref()),
            Some("org.correomqtt.plugins.save-manipulator")
        );
    }

    #[test]
    fn bundled_plugin_ids_keep_file_installed_plugins_non_uninstallable() {
        let repository_json =
            repository_json_with_local_package_source("org.correomqtt.plugins.advanced-validator");
        let state = startup_state_from_current_with_plugins(
            AppConfig::default(),
            HistoryPersistenceSnapshot::default(),
            BTreeMap::new(),
            ScriptPersistenceSnapshot::default(),
            Vec::new(),
            ThemeMode::System,
            vec![repository_json],
            vec!["org.correomqtt.plugins.advanced-validator".to_owned()],
            Vec::new(),
            Vec::new(),
        );
        let plugin = state
            .snapshot
            .plugins
            .plugins
            .iter()
            .find(|plugin| plugin.id == "org.correomqtt.plugins.advanced-validator")
            .expect("bundled plugin should be installed");

        assert_eq!(plugin.source, PluginSource::Bundled);
        assert!(!plugin.can_uninstall());
    }

    #[test]
    fn current_startup_keeps_bundled_plugins_uninstalled_when_setting_is_off() {
        let mut config = AppConfig::default();
        config.settings.install_bundled_plugins = false;

        let state = startup_state_from_current_with_plugins(
            config,
            HistoryPersistenceSnapshot::default(),
            BTreeMap::new(),
            ScriptPersistenceSnapshot::default(),
            Vec::new(),
            ThemeMode::System,
            vec![TEST_REPOSITORY_JSON.to_owned()],
            bundled_plugin_ids(),
            Vec::new(),
            Vec::new(),
        );
        let plugins = &state.snapshot.plugins;

        assert_eq!(plugins.load_state, PluginLoadState::Ready);
        assert_eq!(plugins.marketplace_plugins.len(), 9);
        assert!(plugins.plugins.is_empty());
        assert!(plugins
            .marketplace_plugins
            .iter()
            .all(|plugin| plugin.installed_plugin_id.is_none()));
    }

    fn repository_json_with_local_package_source(plugin_id: &str) -> String {
        let mut value = serde_json::from_str::<serde_json::Value>(TEST_REPOSITORY_JSON).unwrap();
        let plugins = value
            .get_mut("plugins")
            .and_then(serde_json::Value::as_array_mut)
            .unwrap();
        let plugin = plugins
            .iter_mut()
            .find(|plugin| {
                plugin
                    .pointer("/manifest/id")
                    .and_then(serde_json::Value::as_str)
                    == Some(plugin_id)
            })
            .unwrap();
        plugin["install_source"] = serde_json::json!({
            "kind": "local_package",
            "path": format!("plugins/{plugin_id}"),
        });
        serde_json::to_string(&value).unwrap()
    }

    #[test]
    fn startup_uses_hydrated_broker_secret_without_serializing_it() {
        let config = AppConfig {
            built_in_broker: BuiltInBrokerConfig {
                port: "1883".to_owned(),
                credentials_enabled: true,
                username: "broker".to_owned(),
                password: "restart-broker-password".to_owned(),
            },
            ..AppConfig::default()
        };

        let state = startup_state_from_current(
            config,
            HistoryPersistenceSnapshot::default(),
            ScriptPersistenceSnapshot::default(),
            Vec::new(),
            ThemeMode::System,
        );
        let model = crate::AppModel::with_startup_state(state);

        assert_eq!(
            model
                .broker_start_config()
                .and_then(|config| config.password),
            Some("restart-broker-password".to_owned())
        );
        assert!(!serde_json::to_string(model.snapshot())
            .unwrap()
            .contains("restart-broker-password"));
    }

    #[test]
    fn missing_broker_secret_blocks_authenticated_start_without_leaking_secret() {
        let config = AppConfig {
            built_in_broker: BuiltInBrokerConfig {
                port: "1883".to_owned(),
                credentials_enabled: true,
                username: "broker".to_owned(),
                password: String::new(),
            },
            ..AppConfig::default()
        };

        let state = startup_state_from_current(
            config,
            HistoryPersistenceSnapshot::default(),
            ScriptPersistenceSnapshot::default(),
            vec!["Built-in broker credentials are unavailable from secure storage.".to_owned()],
            ThemeMode::System,
        );
        let mut model = crate::AppModel::with_startup_state(state);
        model.apply_command(crate::AppCommand::StartBuiltInBroker);

        assert_eq!(
            model.snapshot().built_in_broker.status,
            crate::BuiltInBrokerStatus::Error
        );
        assert!(model.broker_start_config().is_none());
        assert!(model.snapshot().built_in_broker.logs[0]
            .message
            .contains("Broker configuration is invalid."));
        let serialized = serde_json::to_value(model.snapshot()).unwrap();
        assert!(serialized.pointer("/built_in_broker/password").is_none());
    }
    #[test]
    fn zero_broker_port_blocks_start() {
        let mut model = crate::AppModel::new();
        model.apply_command(crate::AppCommand::UpdateBuiltInBrokerPort("0".to_owned()));
        model.apply_command(crate::AppCommand::StartBuiltInBroker);

        assert_eq!(
            model.snapshot().built_in_broker.status,
            crate::BuiltInBrokerStatus::Error
        );
        assert!(model.broker_start_config().is_none());
        assert!(model.snapshot().built_in_broker.logs[0]
            .message
            .contains("port between 1 and 65535"));
    }

    #[test]
    fn connection_secrets_from_hydrates_matching_imported_values() {
        use correo_storage::current::SecretMaterial;
        let secret = |connection_id: &str, kind, value: &str| ImportedSecret {
            reference: SecretReference {
                connection_id: connection_id.to_owned(),
                kind,
            },
            value: SecretMaterial::new(value),
        };
        let secrets = vec![
            secret("c1", SecretKind::Password, "mqtt-pw"),
            secret("c1", SecretKind::AuthPassword, "ssh-pw"),
            // A secret for another connection must be ignored.
            secret("other", SecretKind::Password, "nope"),
        ];

        let [(password, _), (tls, _), (ssh, _)] = connection_secrets_from(&secrets, "c1");
        assert_eq!(password.expose_for_ui(), "mqtt-pw");
        assert_eq!(tls.expose_for_ui(), "");
        assert_eq!(ssh.expose_for_ui(), "ssh-pw");
    }
}
