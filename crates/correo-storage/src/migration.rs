use crate::current::{
    Auth, ConnectionConfig, HistoryPersistenceSnapshot, ImportedSecret, ScriptPersistenceSnapshot,
    SecretKind as CurrentSecretKind, SecretMaterial, SecretReference, Settings, ThemeSettings,
    TlsSsl,
};
use crate::legacy::passwords::{LegacyPasswordMap, SecretKind as LegacySecretKind};
use crate::legacy::{LegacyConfig, LegacyHookExtension, LegacyHooks, LegacyProfile};
use crate::Result;
use std::path::{Path, PathBuf};

#[path = "migration/config.rs"]
mod config;
#[path = "migration/history.rs"]
mod history;
#[path = "migration/hooks.rs"]
mod hooks;
#[path = "migration/safety.rs"]
mod safety;
#[path = "migration/scripting.rs"]
mod scripting;
#[path = "migration/ui_config.rs"]
mod ui_config;

use config::{migrate_connections, migrate_settings, migrate_theme_settings};
use history::migrate_histories;
use hooks::migrate_legacy_hooks;
use scripting::{migrate_scripts, record_script_unknowns};

pub use safety::{
    MappedLegacyField, MigrationApplier, MigrationApplyOutcome, MigrationBackup,
    MigrationDiagnostic, MigrationDiagnostics,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustPluginManifest {
    pub id: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustPluginState {
    pub manifests: Vec<RustPluginManifest>,
    pub ignored_legacy_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationWarning {
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub warnings: Vec<MigrationWarning>,
    pub unsupported_fields: Vec<UnsupportedLegacyField>,
    pub ignored_java_plugin_state: Vec<IgnoredJavaPluginState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedLegacyField {
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IgnoredJavaPluginState {
    pub kind: IgnoredJavaPluginStateKind,
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IgnoredJavaPluginStateKind {
    JarDirectory,
    ConfigDirectory,
    ProtocolXml,
    Pf4jMetadata,
    HookConfig,
}

#[derive(Clone, Debug)]
pub struct MigrationPreview {
    pub connections: Vec<ConnectionConfig>,
    pub theme_settings: Option<ThemeSettings>,
    pub settings: Settings,
    pub histories: HistoryPersistenceSnapshot,
    pub scripts: ScriptPersistenceSnapshot,
    pub plugin_state: RustPluginState,
    pub warnings: Vec<MigrationWarning>,
    pub report: MigrationReport,
}

impl MigrationPreview {
    pub fn from_legacy_profile(profile: LegacyProfile) -> Result<Self> {
        let mut report = MigrationReport::default();
        record_config_unknowns(&profile.config, &mut report);
        record_hooks_unknowns(&profile.hooks, &mut report);
        for export in &profile.connection_exports {
            record_extra_fields("connectionExport", &export.extra, &mut report);
        }
        let histories = migrate_histories(&profile.histories, &mut report);
        record_script_unknowns(&profile.scripts, &mut report);
        let scripts = migrate_scripts(&profile.scripts, &mut report);
        let theme_settings = migrate_theme_settings(&profile.config);
        let mut settings = migrate_settings(&profile.config);
        let mut connections = migrate_connections(profile.config.connections, &mut report)?;

        if !profile.hooks.outgoing_messages.is_empty()
            || !profile.hooks.incoming_messages.is_empty()
            || !profile.hooks.detail_view_tasks.is_empty()
            || !profile.hooks.message_validators.is_empty()
        {
            let hook_summary =
                migrate_legacy_hooks(&profile.hooks, &mut connections, &mut settings, &mut report);
            if hook_summary.mapped == 0 {
                report.warnings.push(MigrationWarning {
                    code: "legacy_hooks_not_mapped",
                    message: "Legacy Java hook configuration is ignored for Rust plugin state"
                        .to_owned(),
                });
            } else if hook_summary.unsupported > 0 {
                report.warnings.push(MigrationWarning {
                    code: "legacy_hooks_partially_mapped",
                    message: format!(
                        "{} legacy hook workflow assignment(s) migrated; {} hook assignment(s) need manual review",
                        hook_summary.mapped, hook_summary.unsupported
                    ),
                });
            }
            if hook_summary.unsupported > 0 {
                report.ignored_java_plugin_state.push(IgnoredJavaPluginState {
                    kind: IgnoredJavaPluginStateKind::HookConfig,
                    path: PathBuf::from("hooks.json"),
                    reason: format!(
                        "{} Java PF4J hook assignment(s) need manual review after supported workflows were migrated",
                        hook_summary.unsupported
                    ),
                });
            }
        }

        for path in &profile.old_plugin_paths {
            report
                .ignored_java_plugin_state
                .push(IgnoredJavaPluginState {
                    kind: classify_plugin_path(path),
                    path: path.clone(),
                    reason: "Java plugin artifacts are reinitialized from Rust plugin manifests"
                        .to_owned(),
                });
        }

        if !profile.old_plugin_paths.is_empty() {
            report.warnings.push(MigrationWarning {
                code: "legacy_plugins_ignored",
                message: format!(
                    "{} legacy Java plugin artifact(s) ignored",
                    profile.old_plugin_paths.len()
                ),
            });
        }

        let warnings = report.warnings.clone();
        Ok(Self {
            connections,
            theme_settings,
            settings,
            histories,
            scripts,
            plugin_state: RustPluginState::fresh_with_ignored(profile.old_plugin_paths),
            warnings,
            report,
        })
    }
}

impl RustPluginState {
    pub fn fresh_with_ignored(ignored_legacy_paths: Vec<PathBuf>) -> Self {
        Self {
            manifests: vec![
                RustPluginManifest {
                    id: "org.correomqtt.plugins.base64".to_owned(),
                    version: "0.1.0".to_owned(),
                    capabilities: vec![
                        "incoming_transform".to_owned(),
                        "outgoing_transform".to_owned(),
                    ],
                },
                RustPluginManifest {
                    id: "org.correomqtt.plugins.json-format".to_owned(),
                    version: "0.1.0".to_owned(),
                    capabilities: vec!["detail_formatter".to_owned()],
                },
                RustPluginManifest {
                    id: "org.correomqtt.plugins.xml-format".to_owned(),
                    version: "0.1.0".to_owned(),
                    capabilities: vec!["detail_formatter".to_owned()],
                },
                RustPluginManifest {
                    id: "org.correomqtt.plugins.contains-string-validator".to_owned(),
                    version: "0.1.0".to_owned(),
                    capabilities: vec!["validator".to_owned()],
                },
                RustPluginManifest {
                    id: "org.correomqtt.plugins.advanced-validator".to_owned(),
                    version: "0.1.0".to_owned(),
                    capabilities: vec!["validator".to_owned()],
                },
                RustPluginManifest {
                    id: "org.correomqtt.plugins.xml-xsd-validator".to_owned(),
                    version: "0.1.0".to_owned(),
                    capabilities: vec!["validator".to_owned()],
                },
            ],
            ignored_legacy_paths,
        }
    }
}

pub fn connection_secrets<'a>(
    passwords: &'a LegacyPasswordMap,
    connection_id: &str,
) -> Vec<(LegacySecretKind, &'a str)> {
    [
        LegacySecretKind::Password,
        LegacySecretKind::AuthPassword,
        LegacySecretKind::SslKeystorePassword,
    ]
    .into_iter()
    .filter_map(|kind| {
        passwords
            .get(&kind.key(connection_id))
            .map(|secret| (kind, secret.as_str()))
    })
    .collect()
}

pub fn imported_connection_secrets(
    passwords: &LegacyPasswordMap,
    connections: &[ConnectionConfig],
) -> Vec<ImportedSecret> {
    connections
        .iter()
        .flat_map(|connection| {
            connection_secrets(passwords, &connection.id)
                .into_iter()
                .map(|(kind, value)| ImportedSecret {
                    reference: SecretReference {
                        connection_id: connection.id.clone(),
                        kind: current_secret_kind(kind),
                    },
                    value: SecretMaterial::new(value),
                })
        })
        .collect()
}

pub fn configured_secret_slots(connections: &[ConnectionConfig]) -> usize {
    connections
        .iter()
        .map(|connection| {
            usize::from(connection.username.is_some())
                + usize::from(connection.auth == Auth::Password)
                + usize::from(connection.ssl == TlsSsl::Keystore)
        })
        .sum()
}

fn current_secret_kind(kind: LegacySecretKind) -> CurrentSecretKind {
    match kind {
        LegacySecretKind::Password => CurrentSecretKind::Password,
        LegacySecretKind::AuthPassword => CurrentSecretKind::AuthPassword,
        LegacySecretKind::SslKeystorePassword => CurrentSecretKind::SslKeystorePassword,
    }
}

fn record_config_unknowns(config: &LegacyConfig, report: &mut MigrationReport) {
    record_extra_fields("config", &config.extra, report);
    if let Some(settings) = config
        .settings
        .as_ref()
        .and_then(serde_json::Value::as_object)
    {
        record_unknown_value_fields("config.settings", settings, SETTINGS_FIELDS, report);
    }
    for (index, connection) in config.connections.iter().enumerate() {
        record_extra_fields(
            &format!("config.connections[{index}]"),
            &connection.extra,
            report,
        );
    }
}

const SETTINGS_FIELDS: &[&str] = &[
    "useRegexForSearch",
    "useIgnoreCase",
    "savedLocale",
    "currentLocale",
    "searchUpdates",
    "useDefaultRepo",
    "installBundledPlugins",
    "bundledPluginsUrl",
    "pluginRepositories",
    "firstStart",
    "keyringIdentifier",
    "globalUISettings",
    "configCreatedWithCorreoVersion",
];

fn record_hooks_unknowns(hooks: &LegacyHooks, report: &mut MigrationReport) {
    record_extra_fields("hooks", &hooks.extra, report);
    for (index, extension) in hooks.outgoing_messages.iter().enumerate() {
        record_hook_extension_unknowns(
            &format!("hooks.outgoingMessages[{index}]"),
            extension,
            report,
        );
    }
    for (index, extension) in hooks.incoming_messages.iter().enumerate() {
        record_hook_extension_unknowns(
            &format!("hooks.incomingMessages[{index}]"),
            extension,
            report,
        );
    }
    for (task_index, task) in hooks.detail_view_tasks.iter().enumerate() {
        let path = format!("hooks.detailViewTasks[{task_index}]");
        record_extra_fields(&path, &task.extra, report);
        for (extension_index, extension) in task.extensions.iter().enumerate() {
            record_hook_extension_unknowns(
                &format!("{path}.extensions[{extension_index}]"),
                extension,
                report,
            );
        }
    }
    for (validator_index, validator) in hooks.message_validators.iter().enumerate() {
        let path = format!("hooks.messageValidators[{validator_index}]");
        record_extra_fields(&path, &validator.extra, report);
        for (extension_index, extension) in validator.extensions.iter().enumerate() {
            record_hook_extension_unknowns(
                &format!("{path}.extensions[{extension_index}]"),
                extension,
                report,
            );
        }
    }
}

fn record_hook_extension_unknowns(
    path: &str,
    extension: &LegacyHookExtension,
    report: &mut MigrationReport,
) {
    record_extra_fields(path, &extension.extra, report);
}

fn record_extra_fields(
    owner_path: &str,
    extra: &std::collections::BTreeMap<String, serde_json::Value>,
    report: &mut MigrationReport,
) {
    report
        .unsupported_fields
        .extend(extra.keys().map(|field| UnsupportedLegacyField {
            path: format!("{owner_path}.{field}"),
        }));
    report
        .warnings
        .extend(extra.keys().map(|field| MigrationWarning {
            code: "unsupported_legacy_field",
            message: format!("Unsupported legacy field ignored: {owner_path}.{field}"),
        }));
}

fn record_unknown_value_fields(
    owner_path: &str,
    object: &serde_json::Map<String, serde_json::Value>,
    known_fields: &[&str],
    report: &mut MigrationReport,
) {
    for field in object
        .keys()
        .filter(|field| !known_fields.contains(&field.as_str()))
    {
        report.unsupported_fields.push(UnsupportedLegacyField {
            path: format!("{owner_path}.{field}"),
        });
        report.warnings.push(MigrationWarning {
            code: "unsupported_legacy_field",
            message: format!("Unsupported legacy field ignored: {owner_path}.{field}"),
        });
    }
}

fn classify_plugin_path(path: &Path) -> IgnoredJavaPluginStateKind {
    let path = path.to_string_lossy();
    if path.starts_with("plugins/jars") {
        IgnoredJavaPluginStateKind::JarDirectory
    } else if path.starts_with("plugins/config") {
        IgnoredJavaPluginStateKind::ConfigDirectory
    } else if path.ends_with("protocol.xml") {
        IgnoredJavaPluginStateKind::ProtocolXml
    } else {
        IgnoredJavaPluginStateKind::Pf4jMetadata
    }
}
