use serde::{Deserialize, Serialize};

use crate::{PluginConnectionHeaderAction, PluginWindowRow};

#[path = "plugins/repository.rs"]
mod repository;
pub use repository::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSurfaceSnapshot {
    pub active_tab: PluginSurfaceTab,
    pub load_state: PluginLoadState,
    pub plugin_filter: String,
    pub diagnostic_filter: String,
    pub plugins: Vec<PluginRow>,
    #[serde(default)]
    pub marketplace_plugins: Vec<PluginMarketplaceRow>,
    pub selected_plugin_id: String,
    #[serde(default)]
    pub selected_marketplace_plugin_id: String,
    pub selected_diagnostic_id: Option<String>,
    pub feedback: Option<PluginFeedback>,
    pub disable_confirmation: Option<PluginDisableConfirmation>,
    pub hook_editor: Option<PluginHookEditor>,
    #[serde(default)]
    pub open_windows: Vec<PluginWindowRow>,
}

impl PluginSurfaceSnapshot {
    pub fn selected_plugin(&self) -> Option<&PluginRow> {
        self.plugins
            .iter()
            .find(|plugin| plugin.id == self.selected_plugin_id)
    }

    pub fn selected_marketplace_plugin(&self) -> Option<&PluginMarketplaceRow> {
        self.marketplace_plugins
            .iter()
            .find(|plugin| plugin.id == self.selected_marketplace_plugin_id)
    }

    pub fn installed_plugin_for_marketplace(
        &self,
        marketplace_plugin: &PluginMarketplaceRow,
    ) -> Option<&PluginRow> {
        marketplace_plugin
            .installed_plugin_id
            .as_ref()
            .and_then(|plugin_id| self.plugins.iter().find(|plugin| &plugin.id == plugin_id))
    }

    pub fn filtered_plugins(&self) -> Vec<&PluginRow> {
        if self.load_state != PluginLoadState::Ready {
            return Vec::new();
        }

        let filter = self.plugin_filter.trim().to_ascii_lowercase();
        self.plugins
            .iter()
            .filter(|plugin| {
                filter.is_empty()
                    || plugin.name.to_ascii_lowercase().contains(&filter)
                    || plugin.id.to_ascii_lowercase().contains(&filter)
                    || plugin.description.to_ascii_lowercase().contains(&filter)
                    || plugin.provider.to_ascii_lowercase().contains(&filter)
                    || plugin.license.to_ascii_lowercase().contains(&filter)
                    || plugin
                        .capabilities
                        .iter()
                        .any(|capability| capability.label.to_ascii_lowercase().contains(&filter))
            })
            .collect()
    }

    pub fn filtered_marketplace_plugins(&self) -> Vec<&PluginMarketplaceRow> {
        if self.load_state != PluginLoadState::Ready {
            return Vec::new();
        }

        let filter = self.plugin_filter.trim().to_ascii_lowercase();
        self.marketplace_plugins
            .iter()
            .filter(|plugin| {
                filter.is_empty()
                    || plugin.name.to_ascii_lowercase().contains(&filter)
                    || plugin.id.to_ascii_lowercase().contains(&filter)
                    || plugin.description.to_ascii_lowercase().contains(&filter)
                    || plugin.provider.to_ascii_lowercase().contains(&filter)
                    || plugin.license.to_ascii_lowercase().contains(&filter)
                    || plugin.repository.to_ascii_lowercase().contains(&filter)
                    || plugin
                        .capabilities
                        .iter()
                        .any(|capability| capability.label.to_ascii_lowercase().contains(&filter))
            })
            .collect()
    }

    pub fn diagnostics(&self) -> Vec<&PluginDiagnosticRow> {
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.diagnostics.iter())
            .collect()
    }

    pub fn filtered_diagnostics(&self) -> Vec<&PluginDiagnosticRow> {
        let filter = self.diagnostic_filter.trim().to_ascii_lowercase();
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.diagnostics.iter())
            .filter(|diagnostic| {
                filter.is_empty()
                    || diagnostic.plugin_id.to_ascii_lowercase().contains(&filter)
                    || diagnostic.message.to_ascii_lowercase().contains(&filter)
                    || diagnostic.detail.to_ascii_lowercase().contains(&filter)
                    || diagnostic
                        .hook
                        .is_some_and(|hook| hook.label().to_ascii_lowercase().contains(&filter))
            })
            .collect()
    }

    pub fn selected_diagnostic(&self) -> Option<&PluginDiagnosticRow> {
        let selected = self.selected_diagnostic_id.as_ref()?;
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.diagnostics.iter())
            .find(|diagnostic| &diagnostic.id == selected)
    }

    pub fn connection_header_actions(&self) -> Vec<&PluginConnectionHeaderAction> {
        self.plugins
            .iter()
            .filter(|plugin| plugin.enabled && plugin.status == PluginStatus::Active)
            .flat_map(|plugin| plugin.connection_header_actions.iter())
            .collect()
    }

    pub fn active_plugin_ids(&self) -> Vec<String> {
        self.plugins
            .iter()
            .filter(|plugin| plugin.enabled && plugin.status == PluginStatus::Active)
            .map(|plugin| plugin.id.clone())
            .collect()
    }

    pub fn has_connection_workflow_plugins(&self) -> bool {
        self.plugins.iter().any(|plugin| {
            plugin.enabled
                && plugin.status == PluginStatus::Active
                && matches!(
                    plugin.id.as_str(),
                    "org.correomqtt.plugins.contains-string-validator"
                        | "org.correomqtt.plugins.xml-xsd-validator"
                        | "org.correomqtt.plugins.base64"
                        | "org.correomqtt.plugins.save-manipulator"
                        | "org.correomqtt.plugins.zip-manipulator"
                )
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginLoadState {
    Loading,
    Empty,
    #[default]
    Ready,
}

impl PluginLoadState {
    pub fn message(self) -> &'static str {
        match self {
            Self::Loading => "Loading plugin manifests...",
            Self::Empty => {
                "No plugins installed. Bundled replacements can be restored from settings."
            }
            Self::Ready => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRow {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    pub provider: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub installed_path: String,
    pub source: PluginSource,
    pub enabled: bool,
    pub status: PluginStatus,
    pub capabilities: Vec<PluginCapabilityRow>,
    pub config_fields: Vec<PluginConfigField>,
    pub hooks: Vec<PluginHookAssignment>,
    #[serde(default)]
    pub connection_header_actions: Vec<PluginConnectionHeaderAction>,
    pub diagnostics: Vec<PluginDiagnosticRow>,
    pub legacy_note: Option<String>,
}

impl PluginRow {
    pub fn can_uninstall(&self) -> bool {
        self.source == PluginSource::UserManifest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMarketplaceRow {
    pub id: String,
    pub name: String,
    pub version: String,
    pub provider: String,
    pub repository: String,
    pub description: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub location: String,
    pub capabilities: Vec<PluginCapabilityRow>,
    #[serde(default)]
    pub connection_header_actions: Vec<PluginConnectionHeaderAction>,
    #[serde(default)]
    pub install_source: PluginMarketplaceSource,
    pub installed_plugin_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PluginMarketplaceSource {
    Bundled { plugin_id: String },
    LocalPackage { path: String },
    Archive { url: String, sha256: String },
    Unknown,
}

impl Default for PluginMarketplaceSource {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginSurfaceTab {
    #[default]
    Installed,
    Marketplace,
    Configuration,
    Hooks,
    Diagnostics,
}

impl PluginSurfaceTab {
    pub const ALL: [Self; 2] = [Self::Installed, Self::Marketplace];

    pub fn label(self) -> &'static str {
        match self {
            Self::Installed => "Installed",
            Self::Marketplace => "Marketplace",
            Self::Configuration => "Configuration",
            Self::Hooks => "Hooks",
            Self::Diagnostics => "Diagnostics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginSource {
    Bundled,
    UserManifest,
    LegacyJava,
}

impl PluginSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bundled => "Bundled WASM",
            Self::UserManifest => "User manifest",
            Self::LegacyJava => "Legacy Java",
        }
    }
}

impl PluginMarketplaceSource {
    pub fn is_bundled(&self) -> bool {
        matches!(self, Self::Bundled { .. })
    }

    fn plugin_source(&self) -> PluginSource {
        match self {
            Self::Bundled { .. } => PluginSource::Bundled,
            Self::LocalPackage { .. } | Self::Archive { .. } | Self::Unknown => {
                PluginSource::UserManifest
            }
        }
    }

    pub fn location_label(&self) -> String {
        match self {
            Self::Bundled { plugin_id } => format!("bundled://{plugin_id}/plugin.toml"),
            Self::LocalPackage { path } => path.clone(),
            Self::Archive { url, .. } => url.clone(),
            Self::Unknown => "Repository catalog".to_owned(),
        }
    }
}

impl PluginMarketplaceRow {
    pub fn to_installed_plugin(&self) -> PluginRow {
        PluginRow {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            provider: self.provider.clone(),
            license: self.license.clone(),
            location: self.location.clone(),
            origin: self.location.clone(),
            installed_path: String::new(),
            source: self.install_source.plugin_source(),
            enabled: true,
            status: PluginStatus::Active,
            capabilities: self.capabilities.clone(),
            config_fields: Vec::new(),
            hooks: Vec::new(),
            connection_header_actions: self.connection_header_actions.clone(),
            diagnostics: Vec::new(),
            legacy_note: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    Active,
    Disabled,
    NeedsConfig,
    CapabilityDenied,
    LoadError,
    HookFailed,
    UnsupportedLegacy,
}

impl PluginStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Disabled => "Disabled",
            Self::NeedsConfig => "Needs config",
            Self::CapabilityDenied => "Capability denied",
            Self::LoadError => "Load error",
            Self::HookFailed => "Hook failed",
            Self::UnsupportedLegacy => "Unsupported legacy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCapabilityRow {
    pub label: String,
    pub granted: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginConfigField {
    pub key: String,
    pub label: String,
    pub value: String,
    pub saved_value: String,
    pub required: bool,
    pub sensitive: bool,
    pub schema_hint: String,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookAssignment {
    pub hook: PluginHookKind,
    pub enabled: bool,
    pub target: String,
    pub config_json: String,
    pub status: PluginHookStatus,
    pub last_run: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginHookKind {
    IncomingTransform,
    OutgoingTransform,
    Validator,
    DetailTransform,
    DetailFormatter,
}

impl PluginHookKind {
    pub const ALL: [Self; 5] = [
        Self::IncomingTransform,
        Self::OutgoingTransform,
        Self::Validator,
        Self::DetailTransform,
        Self::DetailFormatter,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::IncomingTransform => "Incoming transform",
            Self::OutgoingTransform => "Outgoing transform",
            Self::Validator => "Validator",
            Self::DetailTransform => "Detail transform",
            Self::DetailFormatter => "Detail formatter",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginHookStatus {
    Ready,
    Disabled,
    Denied,
    Failed,
}

impl PluginHookStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Disabled => "Disabled",
            Self::Denied => "Denied",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDiagnosticRow {
    pub id: String,
    pub plugin_id: String,
    pub severity: PluginDiagnosticSeverity,
    pub hook: Option<PluginHookKind>,
    pub message: String,
    pub detail: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl PluginDiagnosticSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginFeedback {
    pub severity: PluginFeedbackSeverity,
    pub message: String,
}

impl PluginFeedback {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginFeedbackSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDisableConfirmation {
    pub plugin_id: String,
    pub plugin_name: String,
    pub active_hooks: Vec<PluginHookKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookEditor {
    pub plugin_id: String,
    pub plugin_name: String,
    pub original: Option<PluginHookDraft>,
    pub draft: PluginHookDraft,
    pub error: Option<String>,
}

impl PluginHookEditor {
    pub fn is_new(&self) -> bool {
        self.original.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookDraft {
    pub hook: PluginHookKind,
    pub enabled: bool,
    pub target: String,
    pub config_json: String,
}

impl From<&PluginHookAssignment> for PluginHookDraft {
    fn from(assignment: &PluginHookAssignment) -> Self {
        Self {
            hook: assignment.hook,
            enabled: assignment.enabled,
            target: assignment.target.clone(),
            config_json: assignment.config_json.clone(),
        }
    }
}
