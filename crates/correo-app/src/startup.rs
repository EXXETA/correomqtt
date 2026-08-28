use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use correo_core::{
    migrate_legacy_built_in_broker, startup_state_from_current_with_plugins,
    startup_state_from_migration, Diagnostic,
};
use correo_core::{StartupState, ThemeMode, WorkbenchSnapshot};
use correo_storage::current::{
    AppConfig, ConfigStore, HistoryPersistenceSnapshot, HistoryStore, ScriptPersistenceSnapshot,
    ScriptStore,
};
use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::MigrationPreview;

use crate::plugins::{load_startup_plugins, StartupPlugins};

#[derive(Debug)]
pub struct LoadedStartup {
    pub state: StartupState,
    pub plugins: StartupPlugins,
}

pub fn load_startup_state(fallback_theme: ThemeMode) -> LoadedStartup {
    for root in correo_storage::current::current_roots() {
        if !root.join("config.json").exists() {
            continue;
        }

        return load_root(&root, fallback_theme.clone()).unwrap_or_else(|error| LoadedStartup {
            state: StartupState::empty(
                fallback_theme,
                Diagnostic::error(format!(
                    "Existing CorreoMQTT config at {} could not be opened: {error}",
                    root.display()
                )),
            ),
            plugins: StartupPlugins::default(),
        });
    }

    for root in correo_storage::current::legacy_roots() {
        if root.join("config.json").exists() {
            return LoadedStartup {
                state: StartupState::legacy_migration_detected(
                    fallback_theme,
                    root.display().to_string(),
                ),
                plugins: StartupPlugins::default(),
            };
        }
    }

    let root = history_root();
    let config = AppConfig::default();
    let plugins = load_startup_plugins(&root, &config);
    let mut state = startup_state_from_current_with_plugins(
        config,
        HistoryPersistenceSnapshot::default(),
        BTreeMap::new(),
        ScriptPersistenceSnapshot::default(),
        Vec::new(),
        fallback_theme,
        plugins.repository_jsons.clone(),
        plugins.bundled_plugin_ids.clone(),
        plugins.installed_plugin_ids.clone(),
        plugins.installed_plugin_paths.clone(),
    );
    state.snapshot.diagnostics.push(
        Diagnostic::info("No existing CorreoMQTT config found; empty workspace ready.").redacted(),
    );
    LoadedStartup { state, plugins }
}

pub fn history_root() -> PathBuf {
    correo_storage::current::config_root()
}

fn load_root(root: &Path, fallback_theme: ThemeMode) -> Result<LoadedStartup, String> {
    match read_current_config(root) {
        Ok((config, warnings)) => {
            let histories = load_current_histories(root, &config)?;
            let workbenches = load_current_workbenches(root, &config)?;
            let scripts = load_current_scripts(root)?;
            let plugins = load_startup_plugins(root, &config);
            let state = startup_state_from_current_with_plugins(
                config,
                histories,
                workbenches,
                scripts,
                warnings,
                fallback_theme,
                plugins.repository_jsons.clone(),
                plugins.bundled_plugin_ids.clone(),
                plugins.installed_plugin_ids.clone(),
                plugins.installed_plugin_paths.clone(),
            );
            Ok(LoadedStartup { state, plugins })
        }
        Err(current_error) => match LegacyProfile::read_from(root) {
            Ok(profile) => {
                let preview = MigrationPreview::from_legacy_profile(profile)
                    .map_err(|error| error.to_string())?;
                Ok(LoadedStartup {
                    state: startup_state_from_migration(preview, fallback_theme),
                    plugins: StartupPlugins::default(),
                })
            }
            Err(legacy_error) => Err(format!(
                "current config error: {current_error}; legacy migration error: {legacy_error}"
            )),
        },
    }
}

fn load_current_scripts(root: &Path) -> Result<ScriptPersistenceSnapshot, String> {
    ScriptStore::new(root)
        .load_snapshot(200)
        .map_err(|error| error.to_string())
}

fn read_current_config(root: &Path) -> Result<(AppConfig, Vec<String>), String> {
    let secret_store = correo_storage::current::default_secret_store();
    read_current_config_with_secret_store(root, secret_store.as_ref())
}

fn read_current_config_with_secret_store(
    root: &Path,
    secret_store: &dyn correo_storage::current::SecretStore,
) -> Result<(AppConfig, Vec<String>), String> {
    let store = ConfigStore::new(root);
    let legacy = store.load().map_err(|error| error.to_string())?;
    let mut config = match migrate_legacy_built_in_broker(&store, secret_store) {
        Ok(migrated) => migrated,
        Err(_) => {
            let mut blocked = legacy;
            blocked.built_in_broker.password.clear();
            return Ok((
                blocked,
                vec!["Built-in broker credential migration is deferred until secure storage is available.".to_owned()],
            ));
        }
    };
    let warnings = if config.built_in_broker.credentials_enabled
        && config.built_in_broker.password.is_empty()
    {
        let reference = store
            .built_in_broker_secret_reference(&config)
            .map_err(|error| error.to_string())?;
        match secret_store.get(&reference) {
            Ok(Some(secret)) => {
                config.built_in_broker.password = secret.expose_for_migration();
                Vec::new()
            }
            _ => {
                vec!["Built-in broker credentials are unavailable from secure storage.".to_owned()]
            }
        }
    } else {
        Vec::new()
    };
    Ok((config, warnings))
}

fn load_current_histories(
    root: &Path,
    config: &AppConfig,
) -> Result<HistoryPersistenceSnapshot, String> {
    let store = HistoryStore::new(root);
    let mut histories = HistoryPersistenceSnapshot::default();
    for connection in &config.connections {
        let history = store
            .load_connection(&connection.id)
            .map_err(|error| error.to_string())?;
        histories.connections.insert(connection.id.clone(), history);
    }
    Ok(histories)
}

fn load_current_workbenches(
    root: &Path,
    config: &AppConfig,
) -> Result<BTreeMap<String, WorkbenchSnapshot>, String> {
    let store = HistoryStore::new(root);
    let mut workbenches = BTreeMap::new();
    for connection in &config.connections {
        let workbench = store
            .load_workbench::<WorkbenchSnapshot>(&connection.id)
            .map_err(|error| error.to_string())?;
        if workbench != WorkbenchSnapshot::default() {
            workbenches.insert(connection.id.clone(), workbench);
        }
    }
    Ok(workbenches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use correo_storage::current::{
        BuiltInBrokerConfig, SecretMaterial, SecretReference, SecretStore,
    };

    struct FixedSecretStore(Option<SecretMaterial>);

    impl SecretStore for FixedSecretStore {
        fn put(
            &self,
            _reference: &SecretReference,
            _value: &SecretMaterial,
        ) -> correo_storage::Result<()> {
            Ok(())
        }

        fn get(
            &self,
            _reference: &SecretReference,
        ) -> correo_storage::Result<Option<SecretMaterial>> {
            Ok(self.0.clone())
        }

        fn delete(&self, _reference: &SecretReference) -> correo_storage::Result<()> {
            Ok(())
        }
    }

    struct MatchingSecretStore {
        reference: SecretReference,
        value: SecretMaterial,
    }

    impl SecretStore for MatchingSecretStore {
        fn put(
            &self,
            _reference: &SecretReference,
            _value: &SecretMaterial,
        ) -> correo_storage::Result<()> {
            Ok(())
        }

        fn get(
            &self,
            reference: &SecretReference,
        ) -> correo_storage::Result<Option<SecretMaterial>> {
            Ok((reference == &self.reference).then(|| self.value.clone()))
        }

        fn delete(&self, _reference: &SecretReference) -> correo_storage::Result<()> {
            Ok(())
        }
    }

    struct UnavailableSecretStore;

    impl SecretStore for UnavailableSecretStore {
        fn put(
            &self,
            _reference: &SecretReference,
            _value: &SecretMaterial,
        ) -> correo_storage::Result<()> {
            Err(correo_storage::StorageError::SecretStore {
                operation: "write",
                reference: "test".to_owned(),
                message: "secure storage unavailable".to_owned(),
            })
        }

        fn get(
            &self,
            _reference: &SecretReference,
        ) -> correo_storage::Result<Option<SecretMaterial>> {
            Ok(None)
        }

        fn delete(&self, _reference: &SecretReference) -> correo_storage::Result<()> {
            Ok(())
        }
    }

    fn temporary_root() -> PathBuf {
        static NEXT_SUFFIX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let suffix = NEXT_SUFFIX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!("correomqtt-startup-{timestamp}-{suffix}"))
    }

    fn persisted_broker(root: &Path) {
        let store = ConfigStore::new(root);
        let config = AppConfig::default();
        let reference = store.built_in_broker_secret_reference(&config).unwrap();
        store
            .save_built_in_broker_with_secret_reference(
                BuiltInBrokerConfig {
                    credentials_enabled: true,
                    username: "broker".to_owned(),
                    ..BuiltInBrokerConfig::default()
                },
                reference,
            )
            .unwrap();
    }

    #[test]
    fn startup_hydrates_persisted_broker_secret_without_rewriting_plaintext() {
        let root = temporary_root();
        persisted_broker(&root);

        let (config, warnings) = read_current_config_with_secret_store(
            &root,
            &FixedSecretStore(Some(SecretMaterial::new("restart-broker-password"))),
        )
        .unwrap();

        assert!(warnings.is_empty());
        assert_eq!(config.built_in_broker.password, "restart-broker-password");
        assert!(!std::fs::read_to_string(root.join("config.json"))
            .unwrap()
            .contains("restart-broker-password"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_persisted_broker_secret_warns_without_enabling_start() {
        let root = temporary_root();
        persisted_broker(&root);

        let (config, warnings) =
            read_current_config_with_secret_store(&root, &FixedSecretStore(None)).unwrap();
        let state = startup_state_from_current_with_plugins(
            config,
            HistoryPersistenceSnapshot::default(),
            BTreeMap::new(),
            ScriptPersistenceSnapshot::default(),
            warnings,
            ThemeMode::System,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let mut model = correo_core::AppModel::with_startup_state(state);
        model.apply_command(correo_core::AppCommand::StartBuiltInBroker);

        assert_eq!(
            model.snapshot().built_in_broker.status,
            correo_core::BuiltInBrokerStatus::Error
        );
        assert!(model.snapshot().built_in_broker.logs[0]
            .message
            .contains("Broker configuration is invalid."));
        assert!(!serde_json::to_string(model.snapshot())
            .unwrap()
            .contains("restart-broker-password"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn startup_does_not_hydrate_an_unmanaged_broker_secret_reference() {
        use correo_storage::current::SecretKind;

        let root = temporary_root();
        let unsafe_reference = SecretReference {
            connection_id: "arbitrary-owner".to_owned(),
            kind: SecretKind::Password,
        };
        let mut config = serde_json::to_value(AppConfig {
            built_in_broker: BuiltInBrokerConfig {
                credentials_enabled: true,
                username: "broker".to_owned(),
                ..BuiltInBrokerConfig::default()
            },
            ..AppConfig::default()
        })
        .unwrap();
        config["built_in_broker"]["password_reference"] =
            serde_json::to_value(&unsafe_reference).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("config.json"),
            serde_json::to_string(&config).unwrap(),
        )
        .unwrap();

        let (config, warnings) = read_current_config_with_secret_store(
            &root,
            &MatchingSecretStore {
                reference: unsafe_reference,
                value: SecretMaterial::new("should-not-hydrate"),
            },
        )
        .unwrap();

        assert!(config.built_in_broker.password.is_empty());
        assert_eq!(
            warnings,
            vec!["Built-in broker credentials are unavailable from secure storage."]
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn deferred_legacy_broker_migration_blocks_authenticated_start() {
        let root = temporary_root();
        let mut config = serde_json::to_value(AppConfig {
            built_in_broker: BuiltInBrokerConfig {
                credentials_enabled: true,
                username: "broker".to_owned(),
                ..BuiltInBrokerConfig::default()
            },
            ..AppConfig::default()
        })
        .unwrap();
        config["built_in_broker"]["password"] = serde_json::json!("legacy-secret");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("config.json"),
            serde_json::to_string(&config).unwrap(),
        )
        .unwrap();

        let (config, warnings) =
            read_current_config_with_secret_store(&root, &UnavailableSecretStore).unwrap();
        let state = startup_state_from_current_with_plugins(
            config,
            HistoryPersistenceSnapshot::default(),
            BTreeMap::new(),
            ScriptPersistenceSnapshot::default(),
            warnings,
            ThemeMode::System,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let mut model = correo_core::AppModel::with_startup_state(state);
        model.apply_command(correo_core::AppCommand::StartBuiltInBroker);

        assert_eq!(
            model.snapshot().built_in_broker.status,
            correo_core::BuiltInBrokerStatus::Error
        );
        assert!(std::fs::read_to_string(root.join("config.json"))
            .unwrap()
            .contains("legacy-secret"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
