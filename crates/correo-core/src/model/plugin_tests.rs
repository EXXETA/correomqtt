use super::AppModel;
use crate::AppCommand;

#[test]
fn plugin_commands_update_config_and_hook_snapshot_state() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::SelectPluginSurfaceTab(
        crate::PluginSurfaceTab::Hooks,
    ));
    assert_eq!(
        model.snapshot().plugins.active_tab,
        crate::PluginSurfaceTab::Hooks
    );

    model.apply_command(AppCommand::UpdatePluginConfigValue {
        plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
        key: "indent".to_owned(),
        value: "4".to_owned(),
    });
    let json_plugin = plugin(&model, "org.correomqtt.plugins.json-format");
    assert_eq!(json_plugin.config_fields[0].value, "4");
    assert_eq!(json_plugin.status, crate::PluginStatus::Active);

    model.apply_command(AppCommand::SetPluginEnabled {
        plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
        enabled: false,
    });
    assert!(model.snapshot().plugins.disable_confirmation.is_some());
    model.apply_command(AppCommand::ConfirmPluginDisable);
    let json_plugin = plugin(&model, "org.correomqtt.plugins.json-format");
    assert!(!json_plugin.enabled);
    assert_eq!(json_plugin.status, crate::PluginStatus::Disabled);
    assert!(json_plugin
        .hooks
        .iter()
        .all(|hook| !hook.enabled && hook.status == crate::PluginHookStatus::Disabled));
}

#[test]
fn plugin_disable_with_active_hooks_can_be_cancelled() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::SetPluginEnabled {
        plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
        enabled: false,
    });
    model.apply_command(AppCommand::CancelPluginDisable);

    let json_plugin = plugin(&model, "org.correomqtt.plugins.json-format");
    assert!(json_plugin.enabled);
    assert!(json_plugin.hooks.iter().any(|hook| hook.enabled));
    assert!(model.snapshot().plugins.disable_confirmation.is_none());
}

#[test]
fn plugin_config_apply_blocks_invalid_json_and_cancel_restores_saved_value() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::UpdatePluginConfigValue {
        plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
        key: "viewer_options".to_owned(),
        value: "{ not json".to_owned(),
    });
    model.apply_command(AppCommand::ApplyPluginConfig {
        plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
    });

    let json_plugin = plugin(&model, "org.correomqtt.plugins.json-format");
    let field = json_plugin
        .config_fields
        .iter()
        .find(|field| field.key == "viewer_options")
        .unwrap();
    assert_eq!(json_plugin.status, crate::PluginStatus::NeedsConfig);
    assert_eq!(field.saved_value, "{\"fold_arrays\":false}");
    assert!(field
        .error
        .as_ref()
        .is_some_and(|error| error.contains("valid JSON")));

    model.apply_command(AppCommand::CancelPluginConfig {
        plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
    });
    let json_plugin = plugin(&model, "org.correomqtt.plugins.json-format");
    let field = json_plugin
        .config_fields
        .iter()
        .find(|field| field.key == "viewer_options")
        .unwrap();
    assert_eq!(field.value, field.saved_value);
    assert!(field.error.is_none());
    assert_eq!(json_plugin.status, crate::PluginStatus::Active);
}

#[test]
fn plugin_hook_editor_add_edit_apply_cancel_and_reset_are_testable() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::StartAddPluginHook {
        plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
    });
    model.apply_command(AppCommand::UpdatePluginHookTarget("draft/#".to_owned()));
    model.apply_command(AppCommand::ResetPluginHookEdit);
    assert_eq!(
        model
            .snapshot()
            .plugins
            .hook_editor
            .as_ref()
            .unwrap()
            .draft
            .target,
        ""
    );
    model.apply_command(AppCommand::CancelPluginHookEdit);
    assert!(model.snapshot().plugins.hook_editor.is_none());

    model.apply_command(AppCommand::StartAddPluginHook {
        plugin_id: "org.correomqtt.plugins.base64".to_owned(),
    });
    model.apply_command(AppCommand::UpdatePluginHookTarget("telemetry/#".to_owned()));
    model.apply_command(AppCommand::UpdatePluginHookConfigJson(
        "{ broken".to_owned(),
    ));
    model.apply_command(AppCommand::ApplyPluginHookEdit);
    assert!(model
        .snapshot()
        .plugins
        .hook_editor
        .as_ref()
        .and_then(|editor| editor.error.as_ref())
        .is_some_and(|error| error.contains("valid JSON")));

    model.apply_command(AppCommand::UpdatePluginHookConfigJson(
        "{\"mode\":\"strict\"}".to_owned(),
    ));
    model.apply_command(AppCommand::ApplyPluginHookEdit);
    let base64 = plugin(&model, "org.correomqtt.plugins.base64");
    assert!(base64
        .hooks
        .iter()
        .any(|hook| hook.hook == crate::PluginHookKind::Validator && hook.target == "telemetry/#"));
    assert!(model.snapshot().plugins.hook_editor.is_none());
}

#[test]
fn plugin_diagnostics_filter_and_required_failure_copy_stay_visible() {
    let mut model = AppModel::default();

    let all_details = model
        .snapshot()
        .plugins
        .diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.detail.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all_details.contains("Publish was blocked"));
    assert!(all_details.contains("original payload remains visible"));
    assert!(all_details.contains("validator rejected the payload"));
    assert!(all_details.contains("fell back to the original raw payload"));

    model.apply_command(AppCommand::SearchPluginDiagnostics(
        "publish was blocked".to_owned(),
    ));
    let filtered = model.snapshot().plugins.filtered_diagnostics();
    assert_eq!(filtered.len(), 2);
    assert!(filtered
        .iter()
        .all(|diagnostic| diagnostic.detail.contains("Publish was blocked")));
}

#[test]
fn wasm_load_error_plugin_cannot_be_enabled() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::SetPluginEnabled {
        plugin_id: "user.wasm-load-error".to_owned(),
        enabled: true,
    });

    let plugin = plugin(&model, "user.wasm-load-error");
    assert!(!plugin.enabled);
    assert_eq!(plugin.status, crate::PluginStatus::LoadError);
    assert!(model
        .snapshot()
        .plugins
        .feedback
        .as_ref()
        .is_some_and(|feedback| feedback.message.contains("WASM load error")));
}

#[test]
fn marketplace_install_and_uninstall_updates_manager_state() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::SelectPluginSurfaceTab(
        crate::PluginSurfaceTab::Marketplace,
    ));
    model.apply_command(AppCommand::SelectMarketplacePlugin(
        "marketplace.schema-validator".to_owned(),
    ));
    model.apply_command(AppCommand::InstallMarketplacePlugin {
        marketplace_plugin_id: "marketplace.schema-validator".to_owned(),
    });

    let installed = plugin(&model, "marketplace.schema-validator");
    assert!(installed.enabled);
    assert_eq!(installed.status, crate::PluginStatus::Active);
    assert_eq!(
        model
            .snapshot()
            .plugins
            .selected_marketplace_plugin()
            .and_then(|plugin| plugin.installed_plugin_id.as_deref()),
        Some("marketplace.schema-validator")
    );

    model.apply_command(AppCommand::UninstallPlugin {
        plugin_id: "marketplace.schema-validator".to_owned(),
    });

    assert!(model
        .snapshot()
        .plugins
        .plugins
        .iter()
        .all(|plugin| plugin.id != "marketplace.schema-validator"));
    assert_eq!(
        model
            .snapshot()
            .plugins
            .selected_marketplace_plugin()
            .and_then(|plugin| plugin.installed_plugin_id.as_deref()),
        None
    );
}

#[test]
fn marketplace_installs_every_plugin_from_local_repository_fixture() {
    let marketplace_plugins = crate::marketplace_rows_from_repository_json(include_str!(
        "../../../correo-plugins/tests/fixtures/repository.json"
    ))
    .unwrap();
    let marketplace_ids = marketplace_plugins
        .iter()
        .map(|plugin| plugin.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        marketplace_plugins
            .iter()
            .filter(|plugin| plugin.install_source.is_bundled())
            .count(),
        8
    );
    assert!(marketplace_plugins
        .iter()
        .find(|plugin| plugin.id == "org.correomqtt.plugins.save-manipulator")
        .is_some_and(|plugin| !plugin.install_source.is_bundled()));
    let mut model = AppModel::default();
    model.snapshot.plugins.marketplace_plugins = marketplace_plugins;

    for plugin_id in &marketplace_ids {
        model.apply_command(AppCommand::SelectMarketplacePlugin(plugin_id.clone()));
        model.apply_command(AppCommand::InstallMarketplacePlugin {
            marketplace_plugin_id: plugin_id.clone(),
        });

        assert!(model
            .snapshot()
            .plugins
            .plugins
            .iter()
            .any(|plugin| &plugin.id == plugin_id));
        assert_eq!(
            model
                .snapshot()
                .plugins
                .selected_marketplace_plugin()
                .and_then(|plugin| plugin.installed_plugin_id.as_deref()),
            Some(plugin_id.as_str())
        );
    }
}

#[test]
fn plugin_denials_legacy_plugins_and_diagnostics_stay_visible() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::SetPluginHookEnabled {
        plugin_id: "user.advanced-validator".to_owned(),
        hook: crate::PluginHookKind::Validator,
        enabled: true,
    });
    let validator = plugin(&model, "user.advanced-validator");
    assert_eq!(validator.hooks[0].status, crate::PluginHookStatus::Denied);
    assert!(model
        .snapshot()
        .plugins
        .feedback
        .as_ref()
        .is_some_and(|feedback| feedback.message.contains("denied capability")));

    let diagnostic_count = model.snapshot().diagnostics.len();
    model.apply_command(AppCommand::SetPluginEnabled {
        plugin_id: "legacy.save-manipulator".to_owned(),
        enabled: true,
    });
    let legacy = plugin(&model, "legacy.save-manipulator");
    assert!(!legacy.enabled);
    assert_eq!(legacy.status, crate::PluginStatus::UnsupportedLegacy);
    assert!(model.snapshot().diagnostics.len() > diagnostic_count);

    model.apply_command(AppCommand::SelectPluginDiagnostic(
        "diag-validator-rejected".to_owned(),
    ));
    assert_eq!(
        model.snapshot().plugins.selected_diagnostic_id.as_deref(),
        Some("diag-validator-rejected")
    );
    model.apply_command(AppCommand::ClearPluginDiagnostics);
    assert!(model.snapshot().plugins.diagnostics().is_empty());
    assert!(model.snapshot().plugins.selected_diagnostic_id.is_none());
}

fn plugin<'a>(model: &'a AppModel, id: &str) -> &'a crate::PluginRow {
    model
        .snapshot()
        .plugins
        .plugins
        .iter()
        .find(|plugin| plugin.id == id)
        .unwrap()
}
