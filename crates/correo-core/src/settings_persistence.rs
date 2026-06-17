use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use correo_storage::current::{
    ConfigStore, ConnectionPluginDirection as StorageConnectionPluginDirection,
    ConnectionPluginWorkflowConfig,
    ConnectionPluginWorkflowKind as StorageConnectionPluginWorkflowKind, PluginStateSettings,
    Settings,
};
use thiserror::Error;

use crate::{
    normalize_keyring_backend, ConnectionPluginDirection, ConnectionPluginWorkflow,
    ConnectionPluginWorkflowKind, GlobalSettingsSnapshot, PluginRepositoryRow, ThemeMode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceCommand {
    Save {
        theme_mode: ThemeMode,
        settings: GlobalSettingsSnapshot,
    },
    SaveConnectionPluginWorkflows {
        connection_id: String,
        workflows: Vec<ConnectionPluginWorkflow>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceEvent {
    Saved,
    Failed { error: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SettingsDispatchError {
    #[error("settings persistence worker is stopped")]
    Stopped,
}

#[derive(Debug)]
pub struct SettingsPersistenceWorker {
    sender: Sender<SettingsPersistenceCommand>,
    events: Receiver<SettingsPersistenceEvent>,
}

impl SettingsPersistenceWorker {
    pub fn start(root: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let store = ConfigStore::new(root.into());

        std::thread::spawn(move || {
            while let Ok(command) = receiver.recv() {
                let event = apply_settings_command(&store, command);
                let _ = events_sender.send(event);
            }
        });

        Self { sender, events }
    }

    pub fn dispatch(
        &self,
        command: SettingsPersistenceCommand,
    ) -> Result<(), SettingsDispatchError> {
        self.sender
            .send(command)
            .map_err(|_| SettingsDispatchError::Stopped)
    }

    pub fn try_recv_event(&self) -> Option<SettingsPersistenceEvent> {
        self.events.try_recv().ok()
    }

    pub fn recv_event_timeout(&self, timeout: Duration) -> Option<SettingsPersistenceEvent> {
        match self.events.recv_timeout(timeout) {
            Ok(event) => Some(event),
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => None,
        }
    }
}

fn apply_settings_command(
    store: &ConfigStore,
    command: SettingsPersistenceCommand,
) -> SettingsPersistenceEvent {
    let result = match command {
        SettingsPersistenceCommand::Save {
            theme_mode,
            settings,
        } => store.save_global_settings(theme_name(&theme_mode), storage_settings(settings)),
        SettingsPersistenceCommand::SaveConnectionPluginWorkflows {
            connection_id,
            workflows,
        } => store.save_connection_plugin_workflows(
            &connection_id,
            workflows.into_iter().map(storage_plugin_workflow).collect(),
        ),
    };

    match result {
        Ok(_) => SettingsPersistenceEvent::Saved,
        Err(error) => SettingsPersistenceEvent::Failed {
            error: error.to_string(),
        },
    }
}

fn storage_plugin_workflow(workflow: ConnectionPluginWorkflow) -> ConnectionPluginWorkflowConfig {
    ConnectionPluginWorkflowConfig {
        plugin_id: workflow.plugin_id,
        enabled: workflow.enabled,
        kind: match workflow.kind {
            ConnectionPluginWorkflowKind::Validator => {
                StorageConnectionPluginWorkflowKind::Validator
            }
            ConnectionPluginWorkflowKind::Manipulator => {
                StorageConnectionPluginWorkflowKind::Manipulator
            }
        },
        direction: match workflow.direction {
            ConnectionPluginDirection::Incoming => StorageConnectionPluginDirection::Incoming,
            ConnectionPluginDirection::Outgoing => StorageConnectionPluginDirection::Outgoing,
            ConnectionPluginDirection::Both => StorageConnectionPluginDirection::Both,
        },
        topic_filter: workflow.topic_filter,
        config: workflow.config,
    }
}

fn storage_settings(snapshot: GlobalSettingsSnapshot) -> Settings {
    let mut settings = Settings::default();
    settings.saved_locale = locale(snapshot.language);
    settings.use_regex_for_search = snapshot.search_use_regex;
    settings.use_ignore_case = snapshot.search_ignore_case;
    settings.reduce_motion = snapshot.reduce_motion;
    settings.search_updates = snapshot.update_checks_enabled;
    settings.use_default_repo = snapshot.use_default_plugin_repository;
    settings.install_bundled_plugins = snapshot.install_bundled_plugins;
    settings.bundled_plugins_url = non_empty(snapshot.bundled_plugins_url);
    settings.plugin_repositories = snapshot
        .plugin_repositories
        .into_iter()
        .filter(|row| !row.url.trim().is_empty())
        .map(repository_entry)
        .collect();
    settings.plugin_states = snapshot
        .plugin_states
        .into_iter()
        .map(|(plugin_id, state)| {
            (
                plugin_id,
                PluginStateSettings {
                    enabled: state.enabled,
                },
            )
        })
        .collect();
    settings.first_start = snapshot.first_start;
    settings.keyring_identifier =
        keyring_identifier(normalize_keyring_backend(snapshot.keyring_backend));
    settings.config_created_with_correo_version = non_unknown(snapshot.config_version);
    settings
}

fn repository_entry(row: PluginRepositoryRow) -> (String, String) {
    (row.id, row.url)
}

fn locale(value: String) -> Option<String> {
    (value != "system").then_some(value)
}

fn keyring_identifier(value: String) -> Option<String> {
    (value != "os").then_some(value)
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn non_unknown(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty() && trimmed != "unknown").then(|| trimmed.to_owned())
}

fn theme_name(mode: &ThemeMode) -> String {
    mode.storage_name().into_owned()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use correo_storage::current::ConfigStore;

    use crate::{
        available_keyring_options, GlobalSettingFlag, GlobalSettingsSnapshot, PluginRepositoryRow,
        SettingsPersistenceCommand, SettingsPersistenceEvent, SettingsPersistenceWorker, ThemeMode,
    };

    #[test]
    fn worker_persists_global_settings_off_the_caller_thread() {
        let temp = tempfile::tempdir().unwrap();
        let worker = SettingsPersistenceWorker::start(temp.path());
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
                settings,
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
}
