use correo_storage::current::{
    ConfigStore, ConnectionPluginDirection, ConnectionPluginWorkflowKind, LabelType,
    PluginHookKind, ScriptExecutionStatus,
};
use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::{
    IgnoredJavaPluginStateKind, MigrationApplier, MigrationDiagnostics, MigrationPreview,
    MigrationWarning,
};
use correo_storage::StorageError;
use std::path::Path;
use std::path::PathBuf;

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

fn legacy_preview() -> MigrationPreview {
    let profile = LegacyProfile::read_from(fixture("legacy_profile")).unwrap();
    MigrationPreview::from_legacy_profile(profile).unwrap()
}

fn seed_existing_target(target: &Path) {
    std::fs::create_dir_all(target.join("scripts")).unwrap();
    std::fs::write(
        target.join("config.json"),
        r#"{"connections":[],"settings":{"saved_locale":"before"}}"#,
    )
    .unwrap();
    std::fs::write(target.join("scripts/original.js"), "logger.info('before');").unwrap();
}

#[test]
fn loads_legacy_profile_fixtures_and_reinitializes_plugins() {
    let profile = LegacyProfile::read_from(fixture("legacy_profile")).unwrap();

    assert_eq!(profile.config.connections.len(), 2);
    assert_eq!(profile.hooks.incoming_messages.len(), 1);
    assert_eq!(profile.hooks.outgoing_messages.len(), 1);
    assert_eq!(
        profile
            .histories
            .publish_topics
            .get("local-broker-01")
            .unwrap()
            .topics,
        ["sensors/temperature", "alerts/status"]
    );
    assert_eq!(
        profile
            .histories
            .subscription_topics
            .get("local-broker-01")
            .unwrap()
            .topics,
        ["sensors/#", "alerts/status"]
    );
    assert_eq!(
        profile
            .histories
            .publish_messages
            .get("local-broker-01")
            .unwrap()
            .messages
            .len(),
        1
    );
    assert_eq!(profile.scripts.len(), 1);
    assert_eq!(
        profile.scripts[0].relative_path,
        Path::new("publish_heartbeat.js")
    );
    assert_eq!(profile.scripts[0].executions.len(), 1);
    assert_eq!(
        profile.scripts[0].executions[0].execution_id.as_deref(),
        Some("execution-001")
    );
    assert_eq!(profile.scripts[0].logs.len(), 1);
    assert_eq!(profile.connection_exports.len(), 1);
    assert_eq!(
        profile.connection_exports[0].connection_config_dtos.len(),
        1
    );
    assert!(profile
        .old_plugin_paths
        .iter()
        .any(|path| path == Path::new("plugins/jars/java-only-plugin.jar")));

    let preview = MigrationPreview::from_legacy_profile(profile).unwrap();
    assert_eq!(preview.settings.saved_locale.as_deref(), Some("de_DE"));
    assert_eq!(preview.settings.current_locale.as_deref(), Some("en_US"));
    assert!(preview.settings.use_regex_for_search);
    assert!(preview.settings.use_ignore_case);
    assert!(!preview.settings.search_updates);
    assert!(!preview.settings.use_default_repo);
    assert!(!preview.settings.install_bundled_plugins);
    assert_eq!(
        preview.settings.keyring_identifier.as_deref(),
        Some("LibSecret")
    );
    assert_eq!(
        preview.settings.plugin_repositories.get("synthetic"),
        Some(&"https://example.invalid/plugins.json".to_owned())
    );
    let local_connection = preview
        .connections
        .iter()
        .find(|connection| connection.id == "local-broker-01")
        .unwrap();
    let connection_ui = local_connection.connection_ui_settings.as_ref().unwrap();
    assert!(connection_ui.show_subscribe);
    assert!(connection_ui.show_publish);
    assert_eq!(connection_ui.main_divider_position, 0.42);
    assert_eq!(connection_ui.publish_detail_divider_position, 0.63);
    assert!(!connection_ui.subscribe_detail_active);
    let publish_labels = &local_connection
        .publish_list_view_config
        .as_ref()
        .unwrap()
        .label_visibility;
    assert_eq!(publish_labels.get(&LabelType::Qos), Some(&true));
    assert_eq!(publish_labels.get(&LabelType::Retained), Some(&false));
    assert_eq!(publish_labels.get(&LabelType::Timestamp), Some(&true));
    let subscribe_labels = &local_connection
        .subscribe_list_view_config
        .as_ref()
        .unwrap()
        .label_visibility;
    assert_eq!(subscribe_labels.get(&LabelType::Qos), Some(&true));
    assert_eq!(subscribe_labels.get(&LabelType::Retained), Some(&false));
    assert!(local_connection.plugin_workflows.iter().any(|workflow| {
        workflow.plugin_id == "org.correomqtt.plugins.base64"
            && workflow.kind == ConnectionPluginWorkflowKind::Manipulator
            && workflow.direction == ConnectionPluginDirection::Outgoing
            && workflow.topic_filter == "#"
    }));
    let contains_string_workflow = local_connection
        .plugin_workflows
        .iter()
        .find(|workflow| workflow.plugin_id == "org.correomqtt.plugins.contains-string-validator")
        .unwrap();
    assert_eq!(
        contains_string_workflow.kind,
        ConnectionPluginWorkflowKind::Validator
    );
    assert_eq!(
        contains_string_workflow.direction,
        ConnectionPluginDirection::Both
    );
    assert_eq!(contains_string_workflow.topic_filter, "alerts/status");
    assert_eq!(contains_string_workflow.config["rules"][0]["text"], "ok");
    let xml_xsd_workflow = local_connection
        .plugin_workflows
        .iter()
        .find(|workflow| workflow.plugin_id == "org.correomqtt.plugins.xml-xsd-validator")
        .unwrap();
    assert_eq!(xml_xsd_workflow.topic_filter, "alerts/status");
    assert_eq!(
        xml_xsd_workflow.config["schema"],
        "/synthetic/schema/note.xsd"
    );
    let xml_hooks = preview
        .settings
        .plugin_hooks
        .get("org.correomqtt.plugins.xml-format")
        .unwrap();
    assert_eq!(xml_hooks.len(), 1);
    assert_eq!(xml_hooks[0].hook, PluginHookKind::DetailFormatter);
    assert_eq!(xml_hooks[0].target, "Format detail");
    assert_eq!(xml_hooks[0].config_json, "{}");
    let global_ui = preview.settings.global_ui_settings.as_ref().unwrap();
    assert_eq!(global_ui.window_width, 1280.0);
    assert_eq!(global_ui.window_height, 800.0);
    let plugin_ids = preview
        .plugin_state
        .manifests
        .iter()
        .map(|manifest| manifest.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        plugin_ids,
        [
            "org.correomqtt.plugins.base64",
            "org.correomqtt.plugins.json-format",
            "org.correomqtt.plugins.xml-format",
            "org.correomqtt.plugins.contains-string-validator",
            "org.correomqtt.plugins.advanced-validator",
            "org.correomqtt.plugins.xml-xsd-validator"
        ]
    );
    assert!(preview
        .plugin_state
        .ignored_legacy_paths
        .iter()
        .any(|path| path == Path::new("plugins/config/java-only-plugin.json")));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_hooks_partially_mapped"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_hook_not_mapped"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_connection_tls_pkcs12_review"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_connection_tls_host_verification_review"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_xml_xsd_schema_path_requires_review"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_plugins_ignored"));
    assert!(preview
        .report
        .unsupported_fields
        .iter()
        .any(|field| field.path == "config.connections[0].futureJavaField"));
    assert!(preview
        .report
        .unsupported_fields
        .iter()
        .any(|field| field.path
            == "scripts.executions.publish_heartbeat.js.execution-001.futureExecutionField"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "unsupported_legacy_field"));
    assert!(preview
        .report
        .ignored_java_plugin_state
        .iter()
        .any(|state| state.kind == IgnoredJavaPluginStateKind::HookConfig));
    assert!(preview
        .report
        .ignored_java_plugin_state
        .iter()
        .any(|state| state.kind == IgnoredJavaPluginStateKind::JarDirectory));
    assert!(preview
        .report
        .ignored_java_plugin_state
        .iter()
        .any(
            |state| state.kind == IgnoredJavaPluginStateKind::Pf4jMetadata
                && state.path == Path::new("plugins/enabled.txt")
        ));
    assert_eq!(preview.scripts.files.len(), 1);
    assert_eq!(preview.scripts.files[0].name, "publish_heartbeat.js");
    let local_history = preview
        .histories
        .connections
        .get("local-broker-01")
        .unwrap();
    assert_eq!(
        local_history.publish_topics.topics,
        ["sensors/temperature", "alerts/status"]
    );
    assert_eq!(
        local_history.subscriptions.topics,
        ["sensors/#", "alerts/status"]
    );
    assert_eq!(local_history.publish_messages.messages.len(), 1);
    assert_eq!(preview.scripts.executions.len(), 1);
    assert_eq!(
        preview.scripts.executions[0].status,
        ScriptExecutionStatus::Succeeded
    );
    assert_eq!(preview.scripts.executions[0].duration_ms, Some(42));
    assert!(!preview.scripts.executions[0].cancelled);
    assert_eq!(preview.scripts.logs.len(), 3);
    assert!(preview
        .scripts
        .logs
        .iter()
        .any(|record| record.message.contains("[REDACTED]")));
    assert!(!preview
        .scripts
        .logs
        .iter()
        .any(|record| record.message.contains("synthetic-log-password")
            || record.message.contains("synthetic-export-password")));
    assert_eq!(preview.warnings, preview.report.warnings);
}

#[test]
fn migration_apply_creates_backup_and_rolls_back_from_temp_fixture() {
    let preview = legacy_preview();
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = temp.path().join("backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let outcome = applier.apply_preview(&preview).unwrap();

    assert!(outcome.backup.id.starts_with("migration-backup-"));
    assert_eq!(outcome.backup.target_root, target);
    assert!(outcome.backup.path.join("config.json").exists());
    assert!(
        std::fs::read_to_string(outcome.backup.path.join("config.json"))
            .unwrap()
            .contains("before")
    );
    assert!(target.join("migration-diagnostics.json").exists());

    let migrated = ConfigStore::new(&target).load().unwrap();
    assert_eq!(migrated.settings.saved_locale.as_deref(), Some("de_DE"));
    assert!(target.join("scripts/publish_heartbeat.js").exists());

    let rollback = applier.rollback(&outcome.backup).unwrap();
    assert!(rollback
        .recovery_steps
        .iter()
        .any(|step| step.contains("Restored migration target")));
    assert!(std::fs::read_to_string(target.join("config.json"))
        .unwrap()
        .contains("before"));
    assert!(target.join("scripts/original.js").exists());
    assert!(!target.join("scripts/publish_heartbeat.js").exists());
}

#[test]
fn migration_apply_restores_backup_when_write_fails() {
    let mut preview = legacy_preview();
    preview.scripts.files[0].relative_path = PathBuf::from("../escape.js");
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = temp.path().join("backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let error = applier.apply_preview(&preview).unwrap_err();

    assert!(matches!(error, StorageError::InvalidScriptFileName(_)));
    assert!(std::fs::read_to_string(target.join("config.json"))
        .unwrap()
        .contains("before"));
    assert!(target.join("scripts/original.js").exists());
    assert!(std::fs::read_dir(backup_root).unwrap().next().is_some());
}

#[test]
fn migration_rollback_refuses_after_target_changes() {
    let preview = legacy_preview();
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = temp.path().join("backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let outcome = applier.apply_preview(&preview).unwrap();
    std::fs::write(target.join("newer-data.txt"), "newer").unwrap();

    let error = applier.rollback(&outcome.backup).unwrap_err();
    assert!(matches!(
        error,
        StorageError::MigrationRollbackSafety { .. }
    ));
    assert!(target.join("newer-data.txt").exists());
}

#[test]
fn migration_rollback_allows_interrupted_marker_without_fingerprint() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = temp.path().join("backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let backup = applier.create_backup().unwrap();
    std::fs::write(target.join("config.json"), r#"{"connections":[]}"#).unwrap();
    std::fs::write(
        target.join(".correo-migration-rollback.json"),
        serde_json::json!({
            "backup_id": backup.id,
            "backup_path": backup.path,
            "state_fingerprint": null
        })
        .to_string(),
    )
    .unwrap();

    applier.rollback(&backup).unwrap();

    assert!(std::fs::read_to_string(target.join("config.json"))
        .unwrap()
        .contains("before"));
    assert!(target.join("scripts/original.js").exists());
}

#[test]
fn migration_diagnostics_capture_fields_and_redact_sensitive_text() {
    let mut preview = legacy_preview();
    preview.report.warnings.push(MigrationWarning {
        code: "synthetic_secret_shape",
        message: "password=synthetic-diagnostic-secret".to_owned(),
    });

    let diagnostics = MigrationDiagnostics::from_preview(&preview, None);

    assert!(diagnostics
        .mapped_fields
        .iter()
        .any(|field| field.legacy_path == "config.connections[].url"));
    assert!(diagnostics
        .unmapped_fields
        .iter()
        .any(|field| field == "config.connections[0].futureJavaField"));
    assert!(diagnostics
        .warnings
        .iter()
        .any(|warning| warning.message.contains("[REDACTED]")));
    let serialized = serde_json::to_string(&diagnostics).unwrap();
    assert!(!serialized.contains("synthetic-diagnostic-secret"));
    assert!(!serialized.contains("synthetic-log-password"));
    assert!(!serialized.contains("synthetic-export-password"));
}
