use crate::capabilities::{CapabilityGrants, HookKind};
use correo_style::{ThemeDefinition, ThemeId};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use thiserror::Error;

pub const SUPPORTED_MANIFEST_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub manifest_version: u16,
    pub id: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub provider: String,
    pub license: String,
    pub compatible_correomqtt: VersionReq,
    pub capabilities: CapabilityGrants,
    #[serde(default)]
    pub entrypoints: Vec<PluginEntrypoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connection_header_actions: Vec<ConnectionHeaderActionContribution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub themes: Vec<ThemeDefinition>,
    #[serde(default)]
    pub config_schema: Option<ConfigSchemaMetadata>,
}

impl PluginManifest {
    pub fn from_toml_str(input: &str) -> Result<Self, ManifestError> {
        let manifest = toml::from_str::<Self>(input)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.manifest_version != SUPPORTED_MANIFEST_VERSION {
            return Err(ManifestError::UnsupportedManifestVersion {
                found: self.manifest_version,
            });
        }

        ensure_non_empty("id", &self.id)?;
        ensure_non_empty("name", &self.name)?;
        ensure_non_empty("description", &self.description)?;
        ensure_non_empty("provider", &self.provider)?;
        ensure_non_empty("license", &self.license)?;

        let mut seen = BTreeSet::new();
        for entrypoint in &self.entrypoints {
            ensure_non_empty("entrypoints.export", &entrypoint.export)?;
            if !self.capabilities.grants_hook(entrypoint.hook) {
                return Err(ManifestError::EntrypointCapabilityMissing {
                    hook: entrypoint.hook,
                });
            }
            if !seen.insert(entrypoint.hook) {
                return Err(ManifestError::DuplicateEntrypoint {
                    hook: entrypoint.hook,
                });
            }
        }

        for theme in &self.themes {
            match &theme.id {
                ThemeId::Plugin(id) if id.starts_with(&format!("{}/", self.id)) => {}
                _ => {
                    return Err(ManifestError::InvalidThemeId {
                        theme_id: theme.id.as_str().into_owned(),
                        plugin_id: self.id.clone(),
                    });
                }
            }
            ensure_non_empty("themes.name", &theme.name)?;
        }

        let mut seen_actions = BTreeSet::new();
        for action in &self.connection_header_actions {
            ensure_non_empty("connection_header_actions.id", &action.id)?;
            ensure_non_empty("connection_header_actions.label", &action.label)?;
            if !seen_actions.insert(&action.id) {
                return Err(ManifestError::DuplicateConnectionHeaderAction {
                    action_id: action.id.clone(),
                });
            }
        }

        Ok(())
    }

    pub fn entrypoint_for(&self, hook: HookKind) -> Option<&PluginEntrypoint> {
        self.entrypoints
            .iter()
            .find(|entrypoint| entrypoint.hook == hook)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginEntrypoint {
    pub hook: HookKind,
    pub export: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionHeaderActionContribution {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub tooltip: String,
    #[serde(default)]
    pub requires_connected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigSchemaMetadata {
    pub schema_version: u16,
    #[serde(default)]
    pub document: Value,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("invalid plugin manifest TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("unsupported plugin manifest version {found}; supported version is 1")]
    UnsupportedManifestVersion { found: u16 },
    #[error("plugin manifest field is empty: {0}")]
    EmptyField(&'static str),
    #[error("entrypoint for {hook:?} requires a matching hook capability grant")]
    EntrypointCapabilityMissing { hook: HookKind },
    #[error("duplicate entrypoint for hook {hook:?}")]
    DuplicateEntrypoint { hook: HookKind },
    #[error("duplicate connection header action {action_id}")]
    DuplicateConnectionHeaderAction { action_id: String },
    #[error("plugin theme id {theme_id} must be namespaced under plugin id {plugin_id}/")]
    InvalidThemeId { theme_id: String, plugin_id: String },
}

fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), ManifestError> {
    if value.trim().is_empty() {
        Err(ManifestError::EmptyField(field))
    } else {
        Ok(())
    }
}
