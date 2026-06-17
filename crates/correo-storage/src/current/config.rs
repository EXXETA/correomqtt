use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Result, StorageError};

use super::Qos;

pub const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Clone, Debug)]
pub struct ConfigStore {
    root: PathBuf,
}

impl ConfigStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load(&self) -> Result<AppConfig> {
        let path = self.path();
        let text = std::fs::read_to_string(&path).map_err(|source| StorageError::Read {
            path: path.clone(),
            source,
        })?;
        serde_json::from_str(&text).map_err(|source| StorageError::Json { path, source })
    }

    pub fn load_or_default(&self) -> Result<AppConfig> {
        if self.path().exists() {
            self.load()
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        std::fs::create_dir_all(&self.root).map_err(|source| StorageError::CreateDir {
            path: self.root.clone(),
            source,
        })?;
        let path = self.path();
        let text = serde_json::to_string_pretty(config).map_err(|source| StorageError::Json {
            path: path.clone(),
            source,
        })?;
        std::fs::write(&path, text).map_err(|source| StorageError::Write { path, source })
    }

    pub fn save_global_settings(
        &self,
        active_theme_name: impl Into<String>,
        mut settings: Settings,
    ) -> Result<AppConfig> {
        let mut config = self.load_or_default()?;
        if settings.global_ui_settings.is_none() {
            settings.global_ui_settings = config.settings.global_ui_settings.clone();
        }
        config.theme_settings = Some(ThemeSettings {
            active_theme: Some(Theme {
                name: Some(active_theme_name.into()),
            }),
        });
        config.settings = settings;
        self.save(&config)?;
        Ok(config)
    }

    pub fn save_connection_plugin_workflows(
        &self,
        connection_id: &str,
        plugin_workflows: Vec<ConnectionPluginWorkflowConfig>,
    ) -> Result<AppConfig> {
        let mut config = self.load_or_default()?;
        if let Some(connection) = config
            .connections
            .iter_mut()
            .find(|connection| connection.id == connection_id)
        {
            connection.plugin_workflows = plugin_workflows;
        }
        self.save(&config)?;
        Ok(config)
    }

    fn path(&self) -> PathBuf {
        self.root.join(CONFIG_FILE_NAME)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
    #[serde(default)]
    pub theme_settings: Option<ThemeSettings>,
    #[serde(default)]
    pub settings: Settings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub port: u16,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub clean_session: bool,
    pub mqtt_version: MqttVersion,
    pub ssl: TlsSsl,
    pub ssl_keystore: Option<String>,
    pub ssl_host_verification: bool,
    pub proxy: Proxy,
    pub ssh_host: Option<String>,
    pub ssh_port: u16,
    pub local_port: Option<u16>,
    pub auth: Auth,
    pub auth_username: Option<String>,
    pub auth_keyfile: Option<String>,
    pub lwt: Lwt,
    pub lwt_topic: Option<String>,
    pub lwt_qos: Option<Qos>,
    pub lwt_retained: bool,
    pub lwt_payload: Option<String>,
    pub connection_ui_settings: Option<ConnectionUiSettings>,
    pub publish_list_view_config: Option<MessageListViewConfig>,
    pub subscribe_list_view_config: Option<MessageListViewConfig>,
    #[serde(default)]
    pub plugin_workflows: Vec<ConnectionPluginWorkflowConfig>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConnectionPluginWorkflowConfig {
    pub plugin_id: String,
    pub enabled: bool,
    pub kind: ConnectionPluginWorkflowKind,
    pub direction: ConnectionPluginDirection,
    pub topic_filter: String,
    pub config: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionPluginWorkflowKind {
    #[default]
    Validator,
    Manipulator,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionPluginDirection {
    #[default]
    Incoming,
    Outgoing,
    Both,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub use_regex_for_search: bool,
    pub use_ignore_case: bool,
    pub reduce_motion: bool,
    pub saved_locale: Option<String>,
    pub current_locale: Option<String>,
    pub search_updates: bool,
    pub use_default_repo: bool,
    pub install_bundled_plugins: bool,
    pub bundled_plugins_url: Option<String>,
    pub plugin_repositories: BTreeMap<String, String>,
    pub plugin_states: BTreeMap<String, PluginStateSettings>,
    pub first_start: bool,
    pub keyring_identifier: Option<String>,
    pub global_ui_settings: Option<GlobalUiSettings>,
    pub config_created_with_correo_version: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            use_regex_for_search: false,
            use_ignore_case: false,
            reduce_motion: false,
            saved_locale: None,
            current_locale: None,
            search_updates: false,
            use_default_repo: true,
            install_bundled_plugins: true,
            bundled_plugins_url: None,
            plugin_repositories: BTreeMap::new(),
            plugin_states: BTreeMap::new(),
            first_start: true,
            keyring_identifier: None,
            global_ui_settings: None,
            config_created_with_correo_version: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginStateSettings {
    pub enabled: bool,
}

impl Default for PluginStateSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub active_theme: Option<Theme>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub name: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GlobalUiSettings {
    pub window_position_x: f64,
    pub window_position_y: f64,
    pub window_width: f64,
    pub window_height: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConnectionUiSettings {
    pub show_subscribe: bool,
    pub show_publish: bool,
    pub main_divider_position: f64,
    pub publish_divider_position: f64,
    pub publish_detail_divider_position: f64,
    pub publish_detail_active: bool,
    pub subscribe_divider_position: f64,
    pub subscribe_detail_divider_position: f64,
    pub subscribe_detail_active: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MessageListViewConfig {
    pub label_visibility: BTreeMap<LabelType, bool>,
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelType {
    Qos,
    Retained,
    Timestamp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MqttVersion {
    Mqtt311,
    Mqtt50,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsSsl {
    Off,
    Keystore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Proxy {
    Off,
    Ssh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Auth {
    Off,
    Password,
    Keyfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lwt {
    Off,
    On,
}
