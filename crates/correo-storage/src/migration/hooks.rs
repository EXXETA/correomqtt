use crate::current::{
    ConnectionConfig, ConnectionPluginDirection, ConnectionPluginWorkflowConfig,
    ConnectionPluginWorkflowKind, PluginHookKind, PluginHookSettings, Settings,
};
use crate::legacy::{LegacyDetailViewTask, LegacyHookExtension, LegacyHooks};
use serde_json::{json, Value};

use super::{MigrationReport, MigrationWarning};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HookMigrationSummary {
    pub mapped: usize,
    pub unsupported: usize,
}

pub(super) fn migrate_legacy_hooks(
    hooks: &LegacyHooks,
    connections: &mut [ConnectionConfig],
    settings: &mut Settings,
    report: &mut MigrationReport,
) -> HookMigrationSummary {
    let mut summary = HookMigrationSummary::default();
    for (index, extension) in hooks.outgoing_messages.iter().enumerate() {
        let path = format!("hooks.outgoingMessages[{index}]");
        migrate_extension(
            extension,
            &path,
            ConnectionPluginWorkflowKind::Manipulator,
            ConnectionPluginDirection::Outgoing,
            "#",
            connections,
            report,
            &mut summary,
        );
    }
    for (index, extension) in hooks.incoming_messages.iter().enumerate() {
        let path = format!("hooks.incomingMessages[{index}]");
        migrate_extension(
            extension,
            &path,
            ConnectionPluginWorkflowKind::Manipulator,
            ConnectionPluginDirection::Incoming,
            "#",
            connections,
            report,
            &mut summary,
        );
    }
    for (validator_index, validator) in hooks.message_validators.iter().enumerate() {
        let topic_filter = validator.topic.as_deref().unwrap_or("#");
        for (extension_index, extension) in validator.extensions.iter().enumerate() {
            let path =
                format!("hooks.messageValidators[{validator_index}].extensions[{extension_index}]");
            migrate_extension(
                extension,
                &path,
                ConnectionPluginWorkflowKind::Validator,
                ConnectionPluginDirection::Both,
                topic_filter,
                connections,
                report,
                &mut summary,
            );
        }
    }
    for (task_index, task) in hooks.detail_view_tasks.iter().enumerate() {
        for (extension_index, extension) in task.extensions.iter().enumerate() {
            migrate_detail_extension(
                task,
                &format!("hooks.detailViewTasks[{task_index}].extensions[{extension_index}]"),
                extension,
                settings,
                report,
                &mut summary,
            );
        }
    }
    summary
}

#[allow(clippy::too_many_arguments)]
fn migrate_extension(
    extension: &LegacyHookExtension,
    path: &str,
    expected_kind: ConnectionPluginWorkflowKind,
    direction: ConnectionPluginDirection,
    topic_filter: &str,
    connections: &mut [ConnectionConfig],
    report: &mut MigrationReport,
    summary: &mut HookMigrationSummary,
) {
    let Some(mapping) = plugin_mapping(extension) else {
        record_unsupported(
            path,
            extension,
            "plugin id has no compatible bundled Rust workflow",
            report,
            summary,
        );
        return;
    };
    if mapping.kind() != expected_kind {
        record_unsupported(
            path,
            extension,
            "legacy hook type does not match a compatible Rust connection workflow",
            report,
            summary,
        );
        return;
    }

    let config = mapping.config(extension);
    if mapping == PluginMapping::XmlXsdValidator {
        record_xml_xsd_schema_path_warning(&config, path, report);
    }

    let workflow = ConnectionPluginWorkflowConfig {
        plugin_id: mapping.current_plugin_id().to_owned(),
        enabled: true,
        kind: mapping.kind(),
        direction,
        topic_filter: topic_filter.to_owned(),
        config,
    };
    for connection in connections {
        connection.plugin_workflows.push(workflow.clone());
        summary.mapped += 1;
    }
}

fn migrate_detail_extension(
    task: &LegacyDetailViewTask,
    path: &str,
    extension: &LegacyHookExtension,
    settings: &mut Settings,
    report: &mut MigrationReport,
    summary: &mut HookMigrationSummary,
) {
    let Some((plugin_id, hook)) = detail_hook_mapping(extension) else {
        record_unsupported(
            path,
            extension,
            "detail view hook plugin id has no compatible bundled Rust hook",
            report,
            summary,
        );
        return;
    };
    settings
        .plugin_hooks
        .entry(plugin_id.to_owned())
        .or_default()
        .push(PluginHookSettings {
            hook,
            enabled: true,
            target: task
                .name
                .as_deref()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or("*")
                .to_owned(),
            config_json: extension
                .config
                .as_ref()
                .cloned()
                .unwrap_or_else(|| json!({}))
                .to_string(),
        });
    summary.mapped += 1;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PluginMapping {
    Base64,
    ZipManipulator,
    SaveManipulator,
    ContainsStringValidator,
    XmlXsdValidator,
}

impl PluginMapping {
    fn current_plugin_id(self) -> &'static str {
        match self {
            Self::Base64 => "org.correomqtt.plugins.base64",
            Self::ZipManipulator => "org.correomqtt.plugins.zip-manipulator",
            Self::SaveManipulator => "org.correomqtt.plugins.save-manipulator",
            Self::ContainsStringValidator => "org.correomqtt.plugins.contains-string-validator",
            Self::XmlXsdValidator => "org.correomqtt.plugins.xml-xsd-validator",
        }
    }

    fn kind(self) -> ConnectionPluginWorkflowKind {
        match self {
            Self::Base64 | Self::ZipManipulator | Self::SaveManipulator => {
                ConnectionPluginWorkflowKind::Manipulator
            }
            Self::ContainsStringValidator | Self::XmlXsdValidator => {
                ConnectionPluginWorkflowKind::Validator
            }
        }
    }

    fn config(self, extension: &LegacyHookExtension) -> Value {
        match self {
            Self::ContainsStringValidator => contains_string_config(extension.config.as_ref()),
            Self::XmlXsdValidator => xml_xsd_config(extension.config.as_ref()),
            Self::SaveManipulator => save_config(extension.config.as_ref()),
            Self::Base64 | Self::ZipManipulator => {
                extension.config.clone().unwrap_or_else(|| json!({}))
            }
        }
    }
}

fn plugin_mapping(extension: &LegacyHookExtension) -> Option<PluginMapping> {
    let plugin_id = extension.plugin_id.as_deref().or(extension.id.as_deref())?;
    let normalized = plugin_id
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '.'], "-");
    if normalized.contains("contains-string") {
        Some(PluginMapping::ContainsStringValidator)
    } else if normalized.contains("xml-xsd") {
        Some(PluginMapping::XmlXsdValidator)
    } else if normalized.contains("base64") {
        Some(PluginMapping::Base64)
    } else if normalized.contains("zip") || normalized.contains("gzip") {
        Some(PluginMapping::ZipManipulator)
    } else if normalized.contains("save") {
        Some(PluginMapping::SaveManipulator)
    } else {
        None
    }
}

fn detail_hook_mapping(extension: &LegacyHookExtension) -> Option<(&'static str, PluginHookKind)> {
    let plugin_id = extension.plugin_id.as_deref().or(extension.id.as_deref())?;
    let normalized = plugin_id
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '.'], "-");
    if normalized.contains("json-format") || normalized.ends_with("json") {
        Some((
            "org.correomqtt.plugins.json-format",
            PluginHookKind::DetailFormatter,
        ))
    } else if normalized.contains("xml-format") {
        Some((
            "org.correomqtt.plugins.xml-format",
            PluginHookKind::DetailFormatter,
        ))
    } else if normalized.contains("zip") || normalized.contains("gzip") {
        Some((
            "org.correomqtt.plugins.zip-manipulator",
            PluginHookKind::DetailTransform,
        ))
    } else {
        None
    }
}

fn contains_string_config(config: Option<&Value>) -> Value {
    let Some(config) = config else {
        return json!({ "rules": [] });
    };
    if config.get("rules").is_some() {
        return config.clone();
    }
    let Some(text) = string_field(config, &["text", "needle", "contains", "value"]) else {
        return json!({ "rules": [] });
    };
    let regex = bool_field(config, &["regex", "useRegex"]);
    json!({ "rules": [{ "text": text, "regex": regex }] })
}

fn xml_xsd_config(config: Option<&Value>) -> Value {
    let Some(config) = config else {
        return json!({ "schema": "" });
    };
    if config.get("schema_text").is_some()
        || config.get("schema_source").is_some()
        || config.get("schema").is_some()
    {
        return config.clone();
    }
    let schema = string_field(
        config,
        &["schemaText", "xsd_path", "xsdPath", "xsd", "schemaPath"],
    )
    .unwrap_or_default();
    if schema.trim_start().starts_with('<') {
        json!({ "schema_text": schema })
    } else {
        json!({ "schema": schema })
    }
}

fn record_xml_xsd_schema_path_warning(config: &Value, path: &str, report: &mut MigrationReport) {
    let Some(schema) = config.get("schema").and_then(Value::as_str) else {
        return;
    };
    if schema.trim().is_empty() || schema.trim_start().starts_with('<') {
        return;
    }
    report.warnings.push(MigrationWarning {
        code: "legacy_xml_xsd_schema_path_requires_review",
        message: format!(
            "Legacy XML/XSD validator {path} references a schema file path; Rust preserves it but the bundled validator requires inline schema text"
        ),
    });
}

fn save_config(config: Option<&Value>) -> Value {
    let Some(config) = config else {
        return json!({ "folder": "" });
    };
    if config.get("folder").is_some() {
        return config.clone();
    }
    let folder = string_field(config, &["saveFolder", "folderPath", "path"]).unwrap_or_default();
    json!({ "folder": folder })
}

fn string_field(config: &Value, fields: &[&str]) -> Option<String> {
    fields
        .iter()
        .find_map(|field| config.get(*field)?.as_str().map(ToOwned::to_owned))
}

fn bool_field(config: &Value, fields: &[&str]) -> bool {
    fields
        .iter()
        .find_map(|field| config.get(*field)?.as_bool())
        .unwrap_or(false)
}

fn record_unsupported(
    path: &str,
    extension: &LegacyHookExtension,
    reason: &'static str,
    report: &mut MigrationReport,
    summary: &mut HookMigrationSummary,
) {
    summary.unsupported += 1;
    let plugin_id = extension
        .plugin_id
        .as_deref()
        .or(extension.id.as_deref())
        .unwrap_or("<missing>");
    report.warnings.push(MigrationWarning {
        code: "legacy_hook_not_mapped",
        message: format!("Legacy hook {path} for {plugin_id} was not mapped: {reason}"),
    });
}
