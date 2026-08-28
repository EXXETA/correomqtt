use crate::{
    PluginCapabilityRow, PluginConfigField, PluginConnectionHeaderAction, PluginDiagnosticRow,
    PluginDiagnosticSeverity, PluginHookAssignment, PluginHookKind, PluginHookStatus,
    PluginLoadState, PluginMarketplaceRow, PluginMarketplaceSource, PluginRow, PluginSource,
    PluginStatus, PluginSurfaceSnapshot, PluginSurfaceTab,
};

pub(super) fn sample_plugins() -> PluginSurfaceSnapshot {
    PluginSurfaceSnapshot {
        active_tab: PluginSurfaceTab::Installed,
        load_state: PluginLoadState::Ready,
        plugin_filter: String::new(),
        diagnostic_filter: String::new(),
        selected_plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
        selected_marketplace_plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
        selected_diagnostic_id: Some("diag-json-ready".to_owned()),
        feedback: None,
        disable_confirmation: None,
        hook_editor: None,
        marketplace_plugins: vec![
            marketplace_json_formatter(),
            marketplace_base64_transform(),
            marketplace_validator_pack(),
        ],
        plugins: vec![
            json_formatter(),
            base64_transform(),
            advanced_validator(),
            system_topic_formatter(),
            user_load_error(),
            legacy_save_plugin(),
        ],
        open_windows: Vec::new(),
    }
}

fn json_formatter() -> PluginRow {
    PluginRow {
        id: "org.correomqtt.plugins.json-format".to_owned(),
        name: "JSON Formatter".to_owned(),
        version: "1.0.0".to_owned(),
        description: "Formats messages into readable JSON".to_owned(),
        provider: "CorreoMQTT".to_owned(),
        license: "GPL".to_owned(),
        location: "bundled://org.correomqtt.plugins.json-format/plugin.toml".to_owned(),
        origin: "bundled://org.correomqtt.plugins.json-format/plugin.toml".to_owned(),
        installed_path: "~/.local/share/correomqtt/plugins/org.correomqtt.plugins.json-format"
            .to_owned(),
        source: PluginSource::Bundled,
        enabled: true,
        status: PluginStatus::Active,
        capabilities: vec![
            cap("Detail formatter", true, "Formats JSON payload details"),
            cap(
                "Detail transform",
                true,
                "Normalizes JSON bytes before display",
            ),
        ],
        config_fields: vec![
            field("indent", "Indent spaces", "2", true, false, "integer"),
            field("sort_keys", "Sort keys", "false", false, false, "boolean"),
            field(
                "viewer_options",
                "Viewer options JSON",
                "{\"fold_arrays\":false}",
                false,
                false,
                "JSON object",
            ),
        ],
        hooks: vec![
            hook(
                PluginHookKind::DetailFormatter,
                true,
                "payload/json",
                PluginHookStatus::Ready,
                "10:25:02",
                "Formatted selected payload",
            ),
            hook(
                PluginHookKind::DetailTransform,
                true,
                "payload/json",
                PluginHookStatus::Ready,
                "10:24:44",
                "Normalized UTF-8 bytes",
            ),
        ],
        connection_header_actions: Vec::new(),
        diagnostics: vec![
            diag(
                "diag-json-ready",
                "org.correomqtt.plugins.json-format",
                PluginDiagnosticSeverity::Info,
                Some(PluginHookKind::DetailFormatter),
                "Bundled formatter initialized",
                "Manifest loaded and compatible entrypoint exports were found.",
                "10:23:58",
            ),
            diag(
                "diag-json-fallback",
                "org.correomqtt.plugins.json-format",
                PluginDiagnosticSeverity::Warning,
                Some(PluginHookKind::DetailFormatter),
                "Formatter fell back to raw payload",
                "Formatting failed for one message, so the detail view fell back to the original raw payload.",
                "10:22:31",
            ),
        ],
        legacy_note: None,
    }
}

fn base64_transform() -> PluginRow {
    PluginRow {
        id: "org.correomqtt.plugins.base64".to_owned(),
        name: "Base64 Transform".to_owned(),
        version: "1.0.0".to_owned(),
        description: "Decodes Base64 encoded messages".to_owned(),
        provider: "CorreoMQTT".to_owned(),
        license: "GPL".to_owned(),
        location: "bundled://org.correomqtt.plugins.base64/plugin.toml".to_owned(),
        origin: "bundled://org.correomqtt.plugins.base64/plugin.toml".to_owned(),
        installed_path: "~/.local/share/correomqtt/plugins/org.correomqtt.plugins.base64"
            .to_owned(),
        source: PluginSource::Bundled,
        enabled: true,
        status: PluginStatus::Active,
        capabilities: vec![
            cap(
                "Incoming transform",
                true,
                "Decodes inbound payload previews",
            ),
            cap("Outgoing transform", true, "Encodes outbound payloads"),
        ],
        config_fields: vec![field(
            "alphabet",
            "Alphabet",
            "standard",
            true,
            false,
            "standard | url_safe",
        )],
        hooks: vec![
            hook(
                PluginHookKind::IncomingTransform,
                true,
                "telemetry/+/encoded",
                PluginHookStatus::Ready,
                "10:19:11",
                "Decoded 8 messages",
            ),
            hook(
                PluginHookKind::OutgoingTransform,
                false,
                "telemetry/+/set",
                PluginHookStatus::Disabled,
                "never",
                "Assignment disabled",
            ),
        ],
        connection_header_actions: Vec::new(),
        diagnostics: vec![
            diag(
                "diag-base64-incoming-failed",
                "org.correomqtt.plugins.base64",
                PluginDiagnosticSeverity::Warning,
                Some(PluginHookKind::IncomingTransform),
                "Incoming transform failed",
                "The transform returned invalid bytes. The original payload remains visible and was not overwritten.",
                "10:19:52",
            ),
            diag(
                "diag-base64-outgoing-blocked",
                "org.correomqtt.plugins.base64",
                PluginDiagnosticSeverity::Error,
                Some(PluginHookKind::OutgoingTransform),
                "Outgoing transform failed",
                "Publish was blocked because the outgoing transform failed before a safe payload could be produced.",
                "10:19:42",
            ),
        ],
        legacy_note: None,
    }
}

fn advanced_validator() -> PluginRow {
    PluginRow {
        id: "user.advanced-validator".to_owned(),
        name: "Advanced Validator".to_owned(),
        version: "0.3.0".to_owned(),
        description: "Advanced message validation with and + or conjunctions".to_owned(),
        provider: "Workspace".to_owned(),
        license: "GPL".to_owned(),
        location: "plugins/advanced-validator/plugin.toml".to_owned(),
        origin: "plugins/advanced-validator/plugin.toml".to_owned(),
        installed_path: "~/.local/share/correomqtt/plugins/user.advanced-validator".to_owned(),
        source: PluginSource::UserManifest,
        enabled: true,
        status: PluginStatus::CapabilityDenied,
        capabilities: vec![
            cap("Validator", true, "Validates message payloads"),
            cap("Network", false, "Denied by host policy"),
        ],
        config_fields: vec![
            field(
                "schema_name",
                "Schema",
                "telemetry-v1",
                true,
                false,
                "string",
            ),
            field(
                "schema_cache_token",
                "Cache token",
                "Stored in keyring",
                false,
                true,
                "secret reference",
            ),
        ],
        hooks: vec![hook(
            PluginHookKind::Validator,
            false,
            "telemetry/#",
            PluginHookStatus::Denied,
            "10:17:22",
            "Network capability denied",
        )],
        connection_header_actions: Vec::new(),
        diagnostics: vec![diag(
            "diag-validator-denied",
            "user.advanced-validator",
            PluginDiagnosticSeverity::Warning,
            Some(PluginHookKind::Validator),
            "Capability request denied",
            "The manifest requested network access. The host denied it, so the validator hook stayed disabled.",
            "10:17:22",
        ), diag(
            "diag-validator-rejected",
            "user.advanced-validator",
            PluginDiagnosticSeverity::Error,
            Some(PluginHookKind::Validator),
            "Validator rejected publish",
            "Publish was blocked because the validator rejected the payload: field `temperature` is required.",
            "10:16:50",
        )],
        legacy_note: None,
    }
}

fn system_topic_formatter() -> PluginRow {
    PluginRow {
        id: "org.correomqtt.plugins.system-topic".to_owned(),
        name: "System Topics".to_owned(),
        version: "1.0.0".to_owned(),
        description: "Adds a connection header action for live $SYS broker metrics.".to_owned(),
        provider: "CorreoMQTT".to_owned(),
        license: "GPL".to_owned(),
        location: "bundled://org.correomqtt.plugins.system-topic/plugin.toml".to_owned(),
        origin: "bundled://org.correomqtt.plugins.system-topic/plugin.toml".to_owned(),
        installed_path: "~/.local/share/correomqtt/plugins/org.correomqtt.plugins.system-topic"
            .to_owned(),
        source: PluginSource::Bundled,
        enabled: true,
        status: PluginStatus::Active,
        capabilities: vec![
            cap("MQTT", true, "Subscribes and unsubscribes through the host"),
            cap("UI", true, "Renders a plugin-owned $SYS window"),
        ],
        config_fields: Vec::new(),
        hooks: Vec::new(),
        connection_header_actions: vec![connection_action(
            "org.correomqtt.plugins.system-topic",
            "system-topics",
            "$SYS",
            "Open system topics",
        )],
        diagnostics: Vec::new(),
        legacy_note: None,
    }
}

fn user_load_error() -> PluginRow {
    PluginRow {
        id: "user.wasm-load-error".to_owned(),
        name: "Broken WASM Import".to_owned(),
        version: "0.1.0".to_owned(),
        description: "Incoming transform manifest failed before its WASM entrypoint could load."
            .to_owned(),
        provider: "Workspace".to_owned(),
        license: "Unspecified".to_owned(),
        location: "plugins/broken-wasm-import/plugin.toml".to_owned(),
        origin: "plugins/broken-wasm-import/plugin.toml".to_owned(),
        installed_path: "~/.local/share/correomqtt/plugins/user.wasm-load-error".to_owned(),
        source: PluginSource::UserManifest,
        enabled: false,
        status: PluginStatus::LoadError,
        capabilities: vec![cap(
            "Incoming transform",
            true,
            "Declared in manifest but entrypoint did not load",
        )],
        config_fields: Vec::new(),
        hooks: vec![hook(
            PluginHookKind::IncomingTransform,
            false,
            "devices/+/raw",
            PluginHookStatus::Failed,
            "never",
            "WASM module failed to instantiate",
        )],
        connection_header_actions: Vec::new(),
        diagnostics: vec![diag(
            "diag-wasm-load-error",
            "user.wasm-load-error",
            PluginDiagnosticSeverity::Error,
            Some(PluginHookKind::IncomingTransform),
            "WASM load error",
            "The plugin manifest was read, but the WASM module could not instantiate because an import was denied.",
            "10:11:30",
        )],
        legacy_note: None,
    }
}

fn legacy_save_plugin() -> PluginRow {
    PluginRow {
        id: "legacy.save-manipulator".to_owned(),
        name: "Save Manipulator".to_owned(),
        version: "legacy".to_owned(),
        description: "Saves message selection to a file".to_owned(),
        provider: "PF4J import".to_owned(),
        license: "GPL".to_owned(),
        location: "~/.correomqtt/plugins/jars/save-manipulator.jar".to_owned(),
        origin: "~/.correomqtt/plugins/jars/save-manipulator.jar".to_owned(),
        installed_path: "~/.correomqtt/plugins/jars/save-manipulator.jar".to_owned(),
        source: PluginSource::LegacyJava,
        enabled: false,
        status: PluginStatus::UnsupportedLegacy,
        capabilities: Vec::new(),
        config_fields: Vec::new(),
        hooks: Vec::new(),
        connection_header_actions: Vec::new(),
        diagnostics: vec![diag(
            "diag-legacy-save-skipped",
            "legacy.save-manipulator",
            PluginDiagnosticSeverity::Warning,
            None,
            "Java plugins were reinitialized",
            "Legacy Java plugin state was left in the backup and Rust/WASM plugin manifests were initialized.",
            "10:10:44",
        )],
        legacy_note: Some(
            "Java/PF4J plugins are not migrated in CorreoMQTT Beta. Old .jar files, PF4J metadata, plugin config, hook config, and protocol.xml were left in the backup."
                .to_owned(),
        ),
    }
}

fn marketplace_json_formatter() -> PluginMarketplaceRow {
    PluginMarketplaceRow {
        id: "org.correomqtt.plugins.json-format".to_owned(),
        name: "JSON Formatter".to_owned(),
        version: "1.0.0".to_owned(),
        provider: "CorreoMQTT".to_owned(),
        repository: "Bundled replacements".to_owned(),
        description: "Formats JSON payloads and normalizes detail bytes.".to_owned(),
        license: "GPL-3.0-or-later".to_owned(),
        location: "bundled://org.correomqtt.plugins.json-format/plugin.toml".to_owned(),
        capabilities: vec![
            cap("Detail formatter", true, "Formats JSON payload details"),
            cap(
                "Detail transform",
                true,
                "Normalizes JSON bytes before display",
            ),
        ],
        connection_header_actions: Vec::new(),
        install_source: PluginMarketplaceSource::Bundled {
            plugin_id: "org.correomqtt.plugins.json-format".to_owned(),
        },
        installed_plugin_id: Some("org.correomqtt.plugins.json-format".to_owned()),
    }
}

fn marketplace_base64_transform() -> PluginMarketplaceRow {
    PluginMarketplaceRow {
        id: "org.correomqtt.plugins.base64".to_owned(),
        name: "Base64 Transform".to_owned(),
        version: "1.0.0".to_owned(),
        provider: "CorreoMQTT".to_owned(),
        repository: "Bundled replacements".to_owned(),
        description: "Decodes inbound payload previews and encodes outbound payloads.".to_owned(),
        license: "GPL-3.0-or-later".to_owned(),
        location: "bundled://org.correomqtt.plugins.base64/plugin.toml".to_owned(),
        capabilities: vec![
            cap(
                "Incoming transform",
                true,
                "Decodes inbound payload previews",
            ),
            cap("Outgoing transform", true, "Encodes outbound payloads"),
        ],
        connection_header_actions: Vec::new(),
        install_source: PluginMarketplaceSource::Bundled {
            plugin_id: "org.correomqtt.plugins.base64".to_owned(),
        },
        installed_plugin_id: Some("org.correomqtt.plugins.base64".to_owned()),
    }
}

fn marketplace_validator_pack() -> PluginMarketplaceRow {
    PluginMarketplaceRow {
        id: "marketplace.schema-validator".to_owned(),
        name: "Schema Validator Pack".to_owned(),
        version: "0.2.0".to_owned(),
        provider: "CorreoMQTT Marketplace".to_owned(),
        repository: "https://example.invalid/plugins.json".to_owned(),
        description: "Adds validator hooks for common MQTT JSON schema checks.".to_owned(),
        license: "GPL-3.0-or-later".to_owned(),
        location: "Repository catalog".to_owned(),
        capabilities: vec![cap("Validator", true, "Validates message payloads")],
        connection_header_actions: Vec::new(),
        install_source: PluginMarketplaceSource::Unknown,
        installed_plugin_id: None,
    }
}

fn cap(label: &str, granted: bool, detail: &str) -> PluginCapabilityRow {
    PluginCapabilityRow {
        label: label.to_owned(),
        granted,
        detail: detail.to_owned(),
    }
}

fn connection_action(
    plugin_id: &str,
    action_id: &str,
    label: &str,
    tooltip: &str,
) -> PluginConnectionHeaderAction {
    PluginConnectionHeaderAction {
        plugin_id: plugin_id.to_owned(),
        action_id: action_id.to_owned(),
        label: label.to_owned(),
        tooltip: tooltip.to_owned(),
        requires_connected: true,
    }
}

fn field(
    key: &str,
    label: &str,
    value: &str,
    required: bool,
    sensitive: bool,
    schema_hint: &str,
) -> PluginConfigField {
    PluginConfigField {
        key: key.to_owned(),
        label: label.to_owned(),
        value: value.to_owned(),
        saved_value: value.to_owned(),
        required,
        sensitive,
        schema_hint: schema_hint.to_owned(),
        valid: !required || !value.trim().is_empty(),
        error: None,
    }
}

fn hook(
    hook: PluginHookKind,
    enabled: bool,
    target: &str,
    status: PluginHookStatus,
    last_run: &str,
    message: &str,
) -> PluginHookAssignment {
    PluginHookAssignment {
        hook,
        enabled,
        target: target.to_owned(),
        config_json: "{}".to_owned(),
        status,
        last_run: last_run.to_owned(),
        message: message.to_owned(),
    }
}

fn diag(
    id: &str,
    plugin_id: &str,
    severity: PluginDiagnosticSeverity,
    hook: Option<PluginHookKind>,
    message: &str,
    detail: &str,
    occurred_at: &str,
) -> PluginDiagnosticRow {
    PluginDiagnosticRow {
        id: id.to_owned(),
        plugin_id: plugin_id.to_owned(),
        severity,
        hook,
        message: message.to_owned(),
        detail: detail.to_owned(),
        occurred_at: occurred_at.to_owned(),
    }
}
