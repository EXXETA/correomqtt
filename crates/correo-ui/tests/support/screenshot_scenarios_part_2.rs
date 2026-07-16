fn transfer_section_for(scenario: Scenario, current: TransferSection) -> TransferSection {
    match scenario {
        Scenario::ImportExport
        | Scenario::ImportChooseFile
        | Scenario::ImportPasswordNeeded
        | Scenario::ImportPasswordError
        | Scenario::ImportReviewWarnings
        | Scenario::ImportCompleteSuccess
        | Scenario::ImportCompleteFailure => TransferSection::Import,
        Scenario::ExportPlain
        | Scenario::ExportEncrypted
        | Scenario::ExportMissingExtension
        | Scenario::ExportInvalidPath
        | Scenario::ExportSuccess
        | Scenario::ExportFailure => TransferSection::Export,
        Scenario::MessageTransfer => TransferSection::Messages,
        _ => current,
    }
}

fn apply_settings_scenario(scenario: Scenario, snapshot: &mut correo_core::AppSnapshot) {
    snapshot.global_settings.selected_section = match scenario {
        Scenario::GlobalSettingsLanguage => SettingsSection::Language,
        Scenario::GlobalSettingsSearch => SettingsSection::Search,
        Scenario::GlobalSettingsKeyring => SettingsSection::Keyring,
        _ => snapshot.global_settings.selected_section,
    };
}

fn apply_plugin_scenario(scenario: Scenario, snapshot: &mut correo_core::AppSnapshot) {
    match scenario {
        Scenario::PluginsLoading => {
            snapshot.plugins.load_state = PluginLoadState::Loading;
            snapshot.plugins.plugins.clear();
            snapshot.plugins.selected_plugin_id.clear();
        }
        Scenario::PluginsEmpty => {
            snapshot.plugins.load_state = PluginLoadState::Empty;
            snapshot.plugins.plugins.clear();
            snapshot.plugins.selected_plugin_id.clear();
        }
        Scenario::PluginsDisableConfirm => {
            snapshot.plugins.selected_plugin_id = "org.correomqtt.plugins.json-format".to_owned();
            snapshot.plugins.disable_confirmation = Some(PluginDisableConfirmation {
                plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
                plugin_name: "JSON Formatter".to_owned(),
                active_hooks: vec![
                    PluginHookKind::DetailFormatter,
                    PluginHookKind::DetailTransform,
                ],
            });
        }
        Scenario::PluginsLoadError => {
            snapshot.plugins.selected_plugin_id = "user.wasm-load-error".to_owned();
            snapshot.plugins.plugin_filter = "wasm".to_owned();
        }
        Scenario::PluginsHookConfigInvalid => {
            snapshot.plugins.active_tab = PluginSurfaceTab::Hooks;
            snapshot.plugins.selected_plugin_id = "org.correomqtt.plugins.json-format".to_owned();
            snapshot.plugins.hook_editor = Some(PluginHookEditor {
                plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
                plugin_name: "JSON Formatter".to_owned(),
                original: None,
                draft: PluginHookDraft {
                    hook: PluginHookKind::IncomingTransform,
                    enabled: true,
                    target: "telemetry/#".to_owned(),
                    config_json: "{ broken".to_owned(),
                },
                error: Some("Config JSON must be valid JSON before Apply.".to_owned()),
            });
        }
        Scenario::PluginsDiagnosticsFiltered => {
            snapshot.plugins.active_tab = PluginSurfaceTab::Diagnostics;
            snapshot.plugins.diagnostic_filter = "publish was blocked".to_owned();
            snapshot.plugins.selected_diagnostic_id =
                Some("diag-base64-outgoing-blocked".to_owned());
        }
        _ => {}
    }
}
