fn storage_qos(value: QosLevel) -> StorageQos {
    match value {
        QosLevel::Zero => StorageQos::AtMostOnce,
        QosLevel::One => StorageQos::AtLeastOnce,
        QosLevel::Two => StorageQos::ExactlyOnce,
    }
}

fn parse_port(value: &str, fallback: u16) -> u16 {
    value
        .trim()
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .unwrap_or(fallback)
}

fn parse_optional_port(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok().filter(|port| *port > 0)
}

fn storage_plugin_workflow(workflow: ConnectionPluginWorkflow) -> ConnectionPluginWorkflowConfig {
    ConnectionPluginWorkflowConfig {
        plugin_id: workflow.plugin_id,
        enabled: workflow.enabled,
        kind: match workflow.kind {
            ConnectionPluginWorkflowKind::Validator => {
                StorageConnectionPluginWorkflowKind::Validator
            }
            ConnectionPluginWorkflowKind::Manipulator => {
                StorageConnectionPluginWorkflowKind::Manipulator
            }
        },
        direction: match workflow.direction {
            ConnectionPluginDirection::Incoming => StorageConnectionPluginDirection::Incoming,
            ConnectionPluginDirection::Outgoing => StorageConnectionPluginDirection::Outgoing,
            ConnectionPluginDirection::Both => StorageConnectionPluginDirection::Both,
        },
        topic_filter: workflow.topic_filter,
        config: workflow.config,
    }
}

fn storage_settings(snapshot: GlobalSettingsSnapshot) -> Settings {
    Settings {
        saved_locale: locale(snapshot.language),
        use_regex_for_search: snapshot.search_use_regex,
        use_ignore_case: snapshot.search_ignore_case,
        reduce_motion: snapshot.reduce_motion,
        search_updates: snapshot.update_checks_enabled,
        use_default_repo: snapshot.use_default_plugin_repository,
        install_bundled_plugins: snapshot.install_bundled_plugins,
        bundled_plugins_url: non_empty(snapshot.bundled_plugins_url),
        plugin_repositories: snapshot
            .plugin_repositories
            .into_iter()
            .filter(|row| !row.url.trim().is_empty())
            .map(repository_entry)
            .collect(),
        plugin_states: snapshot
            .plugin_states
            .into_iter()
            .map(|(plugin_id, state)| {
                (
                    plugin_id,
                    PluginStateSettings {
                        enabled: state.enabled,
                    },
                )
            })
            .collect(),
        plugin_hooks: snapshot
            .plugin_hooks
            .into_iter()
            .map(|(plugin_id, hooks)| {
                (
                    plugin_id,
                    hooks.into_iter().map(storage_plugin_hook).collect(),
                )
            })
            .collect(),
        first_start: snapshot.first_start,
        keyring_identifier: keyring_identifier(normalize_keyring_backend(snapshot.keyring_backend)),
        config_created_with_correo_version: non_unknown(snapshot.config_version),
        ..Default::default()
    }
}

fn storage_plugin_hook(hook: PluginHookSettingsSnapshot) -> PluginHookSettings {
    PluginHookSettings {
        hook: storage_plugin_hook_kind(hook.hook),
        enabled: hook.enabled,
        target: hook.target,
        config_json: hook.config_json,
    }
}

fn storage_plugin_hook_kind(kind: PluginHookKind) -> StoragePluginHookKind {
    match kind {
        PluginHookKind::IncomingTransform => StoragePluginHookKind::IncomingTransform,
        PluginHookKind::OutgoingTransform => StoragePluginHookKind::OutgoingTransform,
        PluginHookKind::Validator => StoragePluginHookKind::Validator,
        PluginHookKind::DetailTransform => StoragePluginHookKind::DetailTransform,
        PluginHookKind::DetailFormatter => StoragePluginHookKind::DetailFormatter,
    }
}

fn repository_entry(row: PluginRepositoryRow) -> (String, String) {
    (row.id, row.url)
}

fn locale(value: String) -> Option<String> {
    (value != "system").then_some(value)
}

fn keyring_identifier(value: String) -> Option<String> {
    (value != "os").then_some(value)
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn non_unknown(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty() && trimmed != "unknown").then(|| trimmed.to_owned())
}

fn theme_name(mode: &ThemeMode) -> String {
    mode.storage_name().into_owned()
}

#[cfg(test)]
#[path = "settings_persistence_tests.rs"]
mod tests;
