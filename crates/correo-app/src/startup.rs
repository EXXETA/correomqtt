use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use correo_core::{
    startup_state_from_current_with_plugins, startup_state_from_migration, Diagnostic,
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
    for root in current_roots() {
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

    for root in legacy_roots() {
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
        Ok(config) => {
            let histories = load_current_histories(root, &config)?;
            let workbenches = load_current_workbenches(root, &config)?;
            let scripts = load_current_scripts(root)?;
            let plugins = load_startup_plugins(root, &config);
            let state = startup_state_from_current_with_plugins(
                config,
                histories,
                workbenches,
                scripts,
                Vec::new(),
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

fn read_current_config(root: &Path) -> Result<AppConfig, String> {
    ConfigStore::new(root)
        .load()
        .map_err(|error| error.to_string())
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

fn current_roots() -> Vec<PathBuf> {
    // The app reads its config from config_root(), so migration discovery
    // only needs to look there.
    dedup_existing_order(vec![correo_storage::current::config_root()])
}

fn legacy_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        roots.push(PathBuf::from(appdata).join("CorreoMqtt"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        roots.push(
            home.join("Library")
                .join("Application Support")
                .join("CorreoMqtt"),
        );
        roots.push(home.join(".correomqtt"));
    }
    roots
}

fn dedup_existing_order(roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();
    for root in roots {
        if !deduped.iter().any(|existing| existing == &root) {
            deduped.push(root);
        }
    }
    deduped
}
