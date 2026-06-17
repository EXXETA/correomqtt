use serde::Deserialize;
use thiserror::Error;

use super::{PluginCapabilityRow, PluginMarketplaceRow, PluginMarketplaceSource};
use crate::PluginConnectionHeaderAction;

const SUPPORTED_REPOSITORY_FORMAT_VERSION: u16 = 1;

pub fn marketplace_rows_from_repository_json(
    input: &str,
) -> Result<Vec<PluginMarketplaceRow>, PluginRepositoryReadError> {
    let repository = serde_json::from_str::<PluginRepositoryDto>(input)?;
    if repository.repository_format_version != SUPPORTED_REPOSITORY_FORMAT_VERSION {
        return Err(PluginRepositoryReadError::UnsupportedFormatVersion {
            found: repository.repository_format_version,
        });
    }

    Ok(repository
        .plugins
        .into_iter()
        .map(|entry| {
            let location = entry.install_source.location_label();
            let plugin_id = entry.manifest.id;
            PluginMarketplaceRow {
                id: plugin_id.clone(),
                name: entry.manifest.name,
                version: entry.manifest.version,
                provider: entry.manifest.provider,
                repository: repository.name.clone(),
                description: entry.manifest.description,
                license: entry.manifest.license,
                location,
                capabilities: capability_rows(entry.manifest.capabilities),
                connection_header_actions: connection_header_actions(
                    &plugin_id,
                    entry.manifest.connection_header_actions,
                ),
                install_source: entry.install_source,
                installed_plugin_id: None,
            }
        })
        .collect())
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PluginRepositoryReadError {
    #[error("invalid plugin repository JSON: {0}")]
    Json(String),
    #[error("unsupported plugin repository format version {found}; supported version is 1")]
    UnsupportedFormatVersion { found: u16 },
}

impl From<serde_json::Error> for PluginRepositoryReadError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct PluginRepositoryDto {
    repository_format_version: u16,
    name: String,
    plugins: Vec<PluginRepositoryEntryDto>,
}

#[derive(Debug, Deserialize)]
struct PluginRepositoryEntryDto {
    manifest: RepositoryManifestDto,
    #[serde(default)]
    install_source: PluginMarketplaceSource,
}

#[derive(Debug, Deserialize)]
struct RepositoryManifestDto {
    id: String,
    name: String,
    version: String,
    description: String,
    provider: String,
    #[serde(default)]
    license: String,
    capabilities: RepositoryCapabilitiesDto,
    #[serde(default)]
    connection_header_actions: Vec<RepositoryConnectionHeaderActionDto>,
}

#[derive(Debug, Deserialize)]
struct RepositoryConnectionHeaderActionDto {
    id: String,
    label: String,
    #[serde(default)]
    tooltip: String,
    #[serde(default)]
    requires_connected: bool,
}

#[derive(Debug, Default, Deserialize)]
struct RepositoryCapabilitiesDto {
    #[serde(default)]
    hooks: Vec<String>,
    #[serde(default)]
    host: RepositoryHostCapabilitiesDto,
}

#[derive(Debug, Default, Deserialize)]
struct RepositoryHostCapabilitiesDto {
    #[serde(default)]
    filesystem: bool,
    #[serde(default)]
    message_save: bool,
    #[serde(default)]
    network: bool,
    #[serde(default)]
    secrets: bool,
    #[serde(default)]
    mqtt: bool,
    #[serde(default)]
    ui: bool,
}

fn connection_header_actions(
    plugin_id: &str,
    actions: Vec<RepositoryConnectionHeaderActionDto>,
) -> Vec<PluginConnectionHeaderAction> {
    actions
        .into_iter()
        .map(|action| PluginConnectionHeaderAction {
            plugin_id: plugin_id.to_owned(),
            action_id: action.id,
            label: action.label,
            tooltip: action.tooltip,
            requires_connected: action.requires_connected,
        })
        .collect()
}

fn capability_rows(capabilities: RepositoryCapabilitiesDto) -> Vec<PluginCapabilityRow> {
    let mut rows = capabilities
        .hooks
        .into_iter()
        .map(|hook| PluginCapabilityRow {
            label: hook_label(&hook).to_owned(),
            granted: true,
            detail: "Declared by the plugin manifest.".to_owned(),
        })
        .collect::<Vec<_>>();

    for (enabled, label) in [
        (capabilities.host.filesystem, "Filesystem"),
        (capabilities.host.message_save, "Message save"),
        (capabilities.host.network, "Network"),
        (capabilities.host.secrets, "Secrets"),
        (capabilities.host.mqtt, "MQTT"),
        (capabilities.host.ui, "UI"),
    ] {
        if enabled {
            rows.push(PluginCapabilityRow {
                label: label.to_owned(),
                granted: true,
                detail: "Requested host surface from the plugin manifest.".to_owned(),
            });
        }
    }

    rows
}

fn hook_label(hook: &str) -> &str {
    match hook {
        "outgoing_message_transform" => "Outgoing transform",
        "incoming_message_transform" => "Incoming transform",
        "message_validator" => "Validator",
        "detail_byte_transform" => "Detail transform",
        "detail_formatter" => "Detail formatter",
        _ => "Plugin hook",
    }
}
