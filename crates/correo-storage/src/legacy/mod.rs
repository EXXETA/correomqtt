pub mod passwords;
mod scripting;

pub use scripting::{
    LegacyScriptExecution, LegacyScriptExecutionError, LegacyScriptLog, ScriptFile,
};

use crate::error::{read_json, Result, StorageError};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyConfig {
    #[serde(default)]
    pub connections: Vec<LegacyConnection>,
    #[serde(default, rename = "themesSettings")]
    pub themes_settings: Option<Value>,
    #[serde(default)]
    pub settings: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyConnection {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub clean_session: bool,
    #[serde(default)]
    pub mqtt_version: Option<String>,
    #[serde(default)]
    pub ssl: Option<String>,
    #[serde(default, alias = "ssl_keystore")]
    pub ssl_keystore: Option<String>,
    #[serde(default, alias = "ssl_keystore_password")]
    pub ssl_keystore_password: Option<String>,
    #[serde(default)]
    pub ssl_host_verification: bool,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default)]
    pub ssh_host: Option<String>,
    #[serde(default)]
    pub ssh_port: Option<u16>,
    #[serde(default)]
    pub local_port: Option<u16>,
    #[serde(default)]
    pub auth: Option<String>,
    #[serde(default)]
    pub auth_username: Option<String>,
    #[serde(default)]
    pub auth_keyfile: Option<String>,
    #[serde(default)]
    pub lwt: Option<String>,
    #[serde(default)]
    pub lwt_topic: Option<String>,
    #[serde(default)]
    pub lwt_qo_s: Option<u8>,
    #[serde(default)]
    pub lwt_retained: bool,
    #[serde(default)]
    pub lwt_payload: Option<String>,
    #[serde(default, alias = "connectionUISettings")]
    pub connection_ui_settings: Option<Value>,
    #[serde(default)]
    pub publish_list_view_config: Option<Value>,
    #[serde(default)]
    pub subscribe_list_view_config: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub clean_session: bool,
    pub mqtt_version: Option<String>,
}

impl TryFrom<LegacyConnection> for ConnectionConfig {
    type Error = StorageError;

    fn try_from(value: LegacyConnection) -> Result<Self> {
        Ok(Self {
            id: require(value.id, "connection", "id")?,
            name: require(value.name, "connection", "name")?,
            host: require(value.url, "connection", "url")?,
            port: value.port.unwrap_or(1883),
            client_id: value.client_id,
            username: value.username,
            clean_session: value.clean_session,
            mqtt_version: value.mqtt_version,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyHooks {
    #[serde(default)]
    pub outgoing_messages: Vec<LegacyHookExtension>,
    #[serde(default)]
    pub incoming_messages: Vec<LegacyHookExtension>,
    #[serde(default)]
    pub detail_view_tasks: Vec<LegacyDetailViewTask>,
    #[serde(default)]
    pub message_validators: Vec<LegacyMessageValidator>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyHookExtension {
    pub id: Option<String>,
    pub plugin_id: Option<String>,
    #[serde(default)]
    pub config: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyDetailViewTask {
    pub name: Option<String>,
    #[serde(default)]
    pub extensions: Vec<LegacyHookExtension>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyMessageValidator {
    pub topic: Option<String>,
    #[serde(default)]
    pub extensions: Vec<LegacyHookExtension>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct TopicHistory {
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct PublishMessageHistory {
    #[serde(default)]
    pub messages: Vec<LegacyMessage>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyMessage {
    pub topic: Option<String>,
    pub payload: Option<String>,
    #[serde(default, alias = "isRetained", alias = "retained")]
    pub is_retained: bool,
    #[serde(default)]
    pub qos: Option<u8>,
    #[serde(default)]
    pub date_time: Option<String>,
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub message_type: Option<String>,
    #[serde(default)]
    pub publish_status: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionExport {
    #[serde(default)]
    pub encryption_type: Option<String>,
    #[serde(default)]
    pub encrypted_data: Option<String>,
    #[serde(default, rename = "connectionConfigDTOS")]
    pub connection_config_dtos: Vec<LegacyConnection>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default)]
pub struct LegacyHistories {
    pub publish_topics: BTreeMap<String, TopicHistory>,
    pub publish_messages: BTreeMap<String, PublishMessageHistory>,
    pub subscription_topics: BTreeMap<String, TopicHistory>,
}

#[derive(Clone, Debug)]
pub struct LegacyProfile {
    pub config: LegacyConfig,
    pub hooks: LegacyHooks,
    pub histories: LegacyHistories,
    pub scripts: Vec<ScriptFile>,
    pub connection_exports: Vec<ConnectionExport>,
    pub old_plugin_paths: Vec<PathBuf>,
}

impl LegacyProfile {
    pub fn read_from(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let config: LegacyConfig = read_json(root.join("config.json"))?;
        Ok(Self {
            histories: read_histories(root, &config.connections)?,
            hooks: read_optional_json(root.join("hooks.json"))?.unwrap_or_default(),
            scripts: scripting::read_scripts(root)?,
            connection_exports: read_connection_exports(root)?,
            old_plugin_paths: find_old_plugin_paths(root)?,
            config,
        })
    }
}

fn read_histories(root: &Path, connections: &[LegacyConnection]) -> Result<LegacyHistories> {
    let mut histories = LegacyHistories::default();
    for connection in connections {
        let Some(id) = connection.id.as_ref() else {
            continue;
        };
        read_optional_history(
            root,
            id,
            "publishHistory.json",
            &mut histories.publish_topics,
        )?;
        read_optional_history(
            root,
            id,
            "subscriptionHistory.json",
            &mut histories.subscription_topics,
        )?;
        if let Some(messages) =
            read_optional_json(root.join(format!("{id}_publishMessageHistory.json")))?
        {
            histories.publish_messages.insert(id.clone(), messages);
        }
    }
    Ok(histories)
}

fn read_optional_history(
    root: &Path,
    id: &str,
    suffix: &str,
    out: &mut BTreeMap<String, TopicHistory>,
) -> Result<()> {
    if let Some(history) = read_optional_json(root.join(format!("{id}_{suffix}")))? {
        out.insert(id.to_owned(), history);
    }
    Ok(())
}

fn read_connection_exports(root: &Path) -> Result<Vec<ConnectionExport>> {
    let export_root = root.join("exports");
    if !export_root.exists() {
        return Ok(Vec::new());
    }
    let mut exports = Vec::new();
    for entry in std::fs::read_dir(&export_root).map_err(|source| StorageError::Read {
        path: export_root.clone(),
        source,
    })? {
        let path = entry
            .map_err(|source| StorageError::Read {
                path: export_root.clone(),
                source,
            })?
            .path();
        if path.extension() == Some(OsStr::new("cqc")) {
            exports.push(read_json(path)?);
        }
    }
    Ok(exports)
}

fn find_old_plugin_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    let plugins_root = root.join("plugins");
    if plugins_root.exists() {
        collect_existing_paths(&plugins_root, root, &mut paths)?;
    }

    let root_protocol = root.join("protocol.xml");
    if root_protocol.exists() {
        collect_existing_paths(&root_protocol, root, &mut paths)?;
    }

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn collect_existing_paths(path: &Path, root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).map_err(|source| StorageError::Read {
            path: path.to_path_buf(),
            source,
        })? {
            collect_existing_paths(
                &entry
                    .map_err(|source| StorageError::Read {
                        path: path.to_path_buf(),
                        source,
                    })?
                    .path(),
                root,
                out,
            )?;
        }
    } else {
        out.push(path.strip_prefix(root).unwrap_or(path).to_path_buf());
    }
    Ok(())
}

fn read_optional_json<T: serde::de::DeserializeOwned>(path: PathBuf) -> Result<Option<T>> {
    if path.exists() {
        read_json(path).map(Some)
    } else {
        Ok(None)
    }
}

fn require(value: Option<String>, record: &'static str, field: &'static str) -> Result<String> {
    value.ok_or(StorageError::MissingField { record, field })
}
