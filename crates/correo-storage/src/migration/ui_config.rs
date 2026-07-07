use crate::current::{ConnectionUiSettings, LabelType, MessageListViewConfig};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{MigrationReport, MigrationWarning, UnsupportedLegacyField};

pub(super) fn connection_ui_settings(
    value: Option<&Value>,
    index: usize,
    report: &mut MigrationReport,
) -> Option<ConnectionUiSettings> {
    let value = value?;
    let path = format!("config.connections[{index}].connectionUISettings");
    let Some(object) = value.as_object() else {
        invalid_value_warning(&path, "expected an object", report);
        return None;
    };
    super::record_unknown_value_fields(&path, object, CONNECTION_UI_SETTINGS_FIELDS, report);

    Some(ConnectionUiSettings {
        show_subscribe: bool_field(object, "showSubscribe", &path, report),
        show_publish: bool_field(object, "showPublish", &path, report),
        main_divider_position: f64_field(object, "mainDividerPosition", &path, report),
        publish_divider_position: f64_field(object, "publishDividerPosition", &path, report),
        publish_detail_divider_position: f64_field(
            object,
            "publishDetailDividerPosition",
            &path,
            report,
        ),
        publish_detail_active: bool_field(object, "publishDetailActive", &path, report),
        subscribe_divider_position: f64_field(object, "subscribeDividerPosition", &path, report),
        subscribe_detail_divider_position: f64_field(
            object,
            "subscribeDetailDividerPosition",
            &path,
            report,
        ),
        subscribe_detail_active: bool_field(object, "subscribeDetailActive", &path, report),
    })
}

pub(super) fn message_list_view_config(
    value: Option<&Value>,
    index: usize,
    field: &'static str,
    report: &mut MigrationReport,
) -> Option<MessageListViewConfig> {
    let value = value?;
    let path = format!("config.connections[{index}].{field}");
    let Some(object) = value.as_object() else {
        invalid_value_warning(&path, "expected an object", report);
        return None;
    };
    super::record_unknown_value_fields(&path, object, MESSAGE_LIST_VIEW_CONFIG_FIELDS, report);

    let mut label_visibility = default_label_visibility();
    if let Some(map_value) = object.get("labelVisibilityMap") {
        let Some(map) = map_value.as_object() else {
            invalid_value_warning(
                &format!("{path}.labelVisibilityMap"),
                "expected an object",
                report,
            );
            return Some(MessageListViewConfig { label_visibility });
        };
        for (label, visible) in map {
            let Some(label_type) = label_type(label) else {
                report.unsupported_fields.push(UnsupportedLegacyField {
                    path: format!("{path}.labelVisibilityMap.{label}"),
                });
                report.warnings.push(MigrationWarning {
                    code: "unsupported_legacy_field",
                    message: format!(
                        "Unsupported legacy field ignored: {path}.labelVisibilityMap.{label}"
                    ),
                });
                continue;
            };
            if let Some(visible) = visible.as_bool() {
                label_visibility.insert(label_type, visible);
            } else {
                invalid_value_warning(
                    &format!("{path}.labelVisibilityMap.{label}"),
                    "expected a boolean",
                    report,
                );
            }
        }
    }
    if let Some(columns_value) = object.get("columns") {
        if let Some(columns) = columns_value.as_array() {
            for column in columns {
                let Some(column) = column.as_str() else {
                    invalid_value_warning(
                        &format!("{path}.columns"),
                        "expected string labels",
                        report,
                    );
                    continue;
                };
                match label_type(column) {
                    Some(label_type) => {
                        label_visibility.insert(label_type, true);
                    }
                    None => report.warnings.push(MigrationWarning {
                        code: "legacy_list_label_unknown",
                        message: format!(
                            "Legacy list label {path}.columns[]={column} could not be mapped"
                        ),
                    }),
                }
            }
        } else {
            invalid_value_warning(&format!("{path}.columns"), "expected an array", report);
        }
    }

    Some(MessageListViewConfig { label_visibility })
}

fn default_label_visibility() -> BTreeMap<LabelType, bool> {
    [
        (LabelType::Qos, false),
        (LabelType::Retained, false),
        (LabelType::Timestamp, false),
    ]
    .into_iter()
    .collect()
}

fn label_type(value: &str) -> Option<LabelType> {
    match value.trim().to_ascii_uppercase().as_str() {
        "QOS" | "QO_S" | "QOSLABEL" => Some(LabelType::Qos),
        "RETAINED" => Some(LabelType::Retained),
        "TIMESTAMP" | "TIME_STAMP" => Some(LabelType::Timestamp),
        _ => None,
    }
}

fn bool_field(
    object: &serde_json::Map<String, Value>,
    field: &'static str,
    path: &str,
    report: &mut MigrationReport,
) -> bool {
    match object.get(field) {
        Some(value) => value.as_bool().unwrap_or_else(|| {
            invalid_value_warning(&format!("{path}.{field}"), "expected a boolean", report);
            false
        }),
        None => false,
    }
}

fn f64_field(
    object: &serde_json::Map<String, Value>,
    field: &'static str,
    path: &str,
    report: &mut MigrationReport,
) -> f64 {
    match object.get(field) {
        Some(value) => value.as_f64().unwrap_or_else(|| {
            invalid_value_warning(&format!("{path}.{field}"), "expected a number", report);
            0.0
        }),
        None => 0.0,
    }
}

fn invalid_value_warning(path: &str, reason: &str, report: &mut MigrationReport) {
    report.warnings.push(MigrationWarning {
        code: "legacy_ui_settings_invalid",
        message: format!("Legacy UI setting {path} ignored: {reason}"),
    });
}

const CONNECTION_UI_SETTINGS_FIELDS: &[&str] = &[
    "showSubscribe",
    "showPublish",
    "mainDividerPosition",
    "publishDividerPosition",
    "publishDetailDividerPosition",
    "publishDetailActive",
    "subscribeDividerPosition",
    "subscribeDetailDividerPosition",
    "subscribeDetailActive",
];

const MESSAGE_LIST_VIEW_CONFIG_FIELDS: &[&str] = &["labelVisibilityMap", "columns"];
