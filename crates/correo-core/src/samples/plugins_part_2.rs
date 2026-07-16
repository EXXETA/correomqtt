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
