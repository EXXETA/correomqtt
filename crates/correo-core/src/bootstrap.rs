use std::collections::BTreeMap;
use std::collections::HashMap;

use correo_mqtt::ConnectionId;
use correo_storage::current::{
    default_secret_store, AppConfig, Auth, BuiltInBrokerConfig, ConnectionConfig,
    ConnectionHistorySnapshot, ConnectionPluginDirection as StorageConnectionPluginDirection,
    ConnectionPluginWorkflowConfig,
    ConnectionPluginWorkflowKind as StorageConnectionPluginWorkflowKind,
    HistoryPersistenceSnapshot, ImportedSecret, Lwt, MqttVersion,
    PluginHookKind as StoragePluginHookKind, PluginHookSettings, Proxy, Qos as StorageQos,
    ScriptPersistenceSnapshot, SecretKind, SecretReference, Settings, ThemeSettings, TlsSsl,
};
use correo_storage::migration::MigrationPreview;

use crate::{
    normalize_keyring_backend, AppSnapshot, BuiltInBrokerSnapshot, ConnectDisabledReason,
    ConnectionBadge, ConnectionPluginDirection, ConnectionPluginWorkflow,
    ConnectionPluginWorkflowKind, ConnectionPluginWorkflowStatus, ConnectionSettingsSnapshot,
    ConnectionState, ConnectionSummary, Diagnostic, GlobalSettingsSnapshot, KeyringState,
    LegacyMigrationStatus, MigrationRecoverySnapshot, PluginHookSettingsSnapshot,
    PluginRepositoryRow, PluginStateSnapshot, PublishHistoryRow, QosLevel, SecretInput,
    SubscribePaneSnapshot, SubscriptionRow, ThemeMode, WorkbenchSnapshot,
};

#[path = "bootstrap_scripts.rs"]
mod bootstrap_scripts;
use bootstrap_scripts::{apply_default_script_connection, script_surface};
#[path = "bootstrap_plugins.rs"]
mod bootstrap_plugins;
use bootstrap_plugins::plugin_surface;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupState {
    pub snapshot: AppSnapshot,
    pub connection_settings: HashMap<ConnectionId, ConnectionSettingsSnapshot>,
    pub storage_connection_ids: HashMap<ConnectionId, String>,
    pub workbenches: HashMap<ConnectionId, WorkbenchSnapshot>,
}

impl StartupState {
    pub fn empty(theme_mode: ThemeMode, diagnostic: Diagnostic) -> Self {
        let mut snapshot = AppSnapshot::empty();
        snapshot.theme_mode = theme_mode;
        snapshot.diagnostics = vec![diagnostic.redacted()];
        Self {
            snapshot,
            connection_settings: HashMap::new(),
            storage_connection_ids: HashMap::new(),
            workbenches: HashMap::new(),
        }
    }

    pub fn legacy_migration_detected(
        theme_mode: ThemeMode,
        legacy_path: impl Into<String>,
    ) -> Self {
        let legacy_path = legacy_path.into();
        let mut snapshot = AppSnapshot::empty();
        snapshot.theme_mode = theme_mode;
        snapshot.migration_recovery = MigrationRecoverySnapshot::detected(legacy_path.clone());
        snapshot.global_settings.legacy_migration.status = LegacyMigrationStatus::Detected;
        snapshot.global_settings.legacy_migration.last_status =
            LegacyMigrationStatus::Detected.label().to_owned();
        snapshot.global_settings.legacy_migration.legacy_path_hint = Some(legacy_path.clone());
        snapshot.diagnostics = vec![Diagnostic::info(format!(
            "Legacy CorreoMQTT data detected at {legacy_path}; migration is waiting for user choice."
        ))
        .redacted()];
        Self {
            snapshot,
            connection_settings: HashMap::new(),
            storage_connection_ids: HashMap::new(),
            workbenches: HashMap::new(),
        }
    }
}

pub fn startup_state_from_current(
    config: AppConfig,
    histories: HistoryPersistenceSnapshot,
    scripts: ScriptPersistenceSnapshot,
    warnings: Vec<String>,
    fallback_theme: ThemeMode,
) -> StartupState {
    startup_state_from_current_with_plugins(
        config,
        histories,
        BTreeMap::new(),
        scripts,
        warnings,
        fallback_theme,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub fn startup_state_from_current_with_workbenches(
    config: AppConfig,
    histories: HistoryPersistenceSnapshot,
    persisted_workbenches: BTreeMap<String, WorkbenchSnapshot>,
    scripts: ScriptPersistenceSnapshot,
    warnings: Vec<String>,
    fallback_theme: ThemeMode,
) -> StartupState {
    startup_state_from_current_with_plugins(
        config,
        histories,
        persisted_workbenches,
        scripts,
        warnings,
        fallback_theme,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub fn startup_state_from_current_with_plugins(
    config: AppConfig,
    histories: HistoryPersistenceSnapshot,
    persisted_workbenches: BTreeMap<String, WorkbenchSnapshot>,
    scripts: ScriptPersistenceSnapshot,
    warnings: Vec<String>,
    fallback_theme: ThemeMode,
    plugin_repository_jsons: Vec<String>,
    bundled_plugin_ids: Vec<String>,
    installed_plugin_ids: Vec<String>,
    installed_plugin_paths: Vec<(String, String)>,
) -> StartupState {
    let theme_mode = theme_mode(config.theme_settings.as_ref()).unwrap_or(fallback_theme);
    let mut snapshot = AppSnapshot::empty();
    let mut connection_settings = HashMap::new();
    let mut storage_connection_ids = HashMap::new();
    let mut workbenches = HashMap::new();
    let mut mapped = Vec::new();

    for connection in &config.connections {
        let id = ConnectionId::new();
        let history = histories.connections.get(&connection.id);
        mapped.push(summary(id, connection, history));
        connection_settings.insert(id, settings_snapshot(connection, &warnings, None));
        storage_connection_ids.insert(id, connection.id.clone());
        let workbench = persisted_workbenches
            .get(&connection.id)
            .cloned()
            .or_else(|| history.map(workbench_from_history))
            .unwrap_or_default();
        let mut workbench = workbench;
        normalize_publish_history_ids(&mut workbench);
        workbenches.insert(id, workbench);
    }

    snapshot.connection_count = mapped.len();
    snapshot.selected_connection = mapped.first().map(|connection| connection.id);
    snapshot.connections = mapped;
    snapshot.theme_mode = theme_mode;
    snapshot.built_in_broker = built_in_broker(&config.built_in_broker);
    snapshot.global_settings = global_settings(&config.settings);
    snapshot.plugins = plugin_surface(
        config.settings.install_bundled_plugins,
        &plugin_repository_jsons,
        &bundled_plugin_ids,
        &installed_plugin_ids,
        &installed_plugin_paths,
        &config.settings.plugin_states,
        &config.settings.plugin_hooks,
    );
    snapshot.scripts = script_surface(&scripts);
    snapshot.diagnostics = warnings
        .into_iter()
        .map(|warning| Diagnostic::warning(warning).redacted())
        .collect();

    if let Some(selected) = snapshot.selected_connection {
        if let Some(settings) = connection_settings.get(&selected) {
            snapshot.connection_settings = settings.clone();
        }
        snapshot.workbench = workbenches.get(&selected).cloned().unwrap_or_default();
    }
    apply_default_script_connection(&mut snapshot);

    StartupState {
        snapshot,
        connection_settings,
        storage_connection_ids,
        workbenches,
    }
}

pub fn startup_state_from_migration(
    preview: MigrationPreview,
    fallback_theme: ThemeMode,
) -> StartupState {
    let warnings = preview
        .warnings
        .iter()
        .map(|warning| warning.message.clone())
        .collect();
    startup_state_from_current(
        AppConfig {
            connections: preview.connections,
            theme_settings: preview.theme_settings,
            settings: preview.settings,
            built_in_broker: BuiltInBrokerConfig::default(),
        },
        preview.histories,
        preview.scripts,
        warnings,
        fallback_theme,
    )
}

fn built_in_broker(config: &BuiltInBrokerConfig) -> BuiltInBrokerSnapshot {
    BuiltInBrokerSnapshot {
        port: config.port.clone(),
        credentials_enabled: config.credentials_enabled,
        username: config.username.clone(),
        password: config.password.clone(),
        ..BuiltInBrokerSnapshot::default()
    }
}

fn summary(
    id: ConnectionId,
    connection: &ConnectionConfig,
    history: Option<&ConnectionHistorySnapshot>,
) -> ConnectionSummary {
    ConnectionSummary {
        id,
        name: connection.name.clone(),
        endpoint: format!("{}:{}", connection.url, connection.port),
        mqtt_version: mqtt_label(connection.mqtt_version).to_owned(),
        badges: badges(connection),
        active_plugin_workflows: connection
            .plugin_workflows
            .iter()
            .any(|workflow| workflow.enabled),
        immutable: false,
        state: ConnectionState::Disconnected,
        disabled_reason: connection
            .url
            .trim()
            .is_empty()
            .then_some(ConnectDisabledReason::MissingHost),
        recent_subscriptions: history
            .map(|history| history.subscriptions.topics.len())
            .unwrap_or_default(),
        recent_messages: history
            .map(|history| history.publish_messages.messages.len())
            .unwrap_or_default(),
        last_activity: history
            .map(history_activity)
            .unwrap_or_else(|| "Ready".to_owned()),
    }
}

pub(crate) fn settings_snapshot(
    connection: &ConnectionConfig,
    warnings: &[String],
    secret_override: Option<&[ImportedSecret]>,
) -> ConnectionSettingsSnapshot {
    let valid = !connection.name.trim().is_empty() && !connection.url.trim().is_empty();
    // A freshly imported connection carries its secrets in memory: use them so
    // the snapshot is usable before the async keyring write lands, instead of
    // reading the not-yet-written keyring.
    let [(password, password_keyring_state), (tls_keystore_password, tls_keyring_state), (ssh_password, ssh_keyring_state)] =
        match secret_override {
            Some(secrets) => connection_secrets_from(secrets, &connection.id),
            None => connection_secrets(&connection.id),
        };
    let password_status = password_status(connection, &password).to_owned();
    let tls_password_status = tls_password_status(connection, &tls_keystore_password).to_owned();
    let ssh_password_status = ssh_password_status(connection, &ssh_password).to_owned();
    ConnectionSettingsSnapshot {
        internal_id: connection.id.clone(),
        profile_name: connection.name.clone(),
        host: connection.url.clone(),
        port: connection.port.to_string(),
        mqtt_version: mqtt_label(connection.mqtt_version).to_owned(),
        clean_session: connection.clean_session,
        client_id: connection.client_id.clone().unwrap_or_default(),
        username: connection.username.clone().unwrap_or_default(),
        password,
        password_status,
        auth_mode: auth_label(connection).to_owned(),
        tls_mode: tls_label(connection).to_owned(),
        tls_store: connection.ssl_keystore.clone().unwrap_or_default(),
        tls_keystore_password,
        tls_password_status,
        tls_host_verification: connection.ssl_host_verification,
        proxy_mode: proxy_label(connection).to_owned(),
        ssh_host: connection.ssh_host.clone().unwrap_or_default(),
        ssh_port: connection.ssh_port.to_string(),
        local_mqtt_port: connection
            .local_port
            .map(|port| port.to_string())
            .unwrap_or_default(),
        auth_username: connection.auth_username.clone().unwrap_or_default(),
        ssh_password,
        ssh_password_status,
        ssh_key_file: connection.auth_keyfile.clone().unwrap_or_default(),
        lwt_enabled: connection.lwt == Lwt::On,
        lwt_topic: connection.lwt_topic.clone().unwrap_or_default(),
        lwt_qos: connection.lwt_qos.map(qos).unwrap_or(QosLevel::One),
        lwt_retained: connection.lwt_retained,
        lwt_payload: connection.lwt_payload.clone().unwrap_or_default(),
        dirty: false,
        valid,
        save_disabled_reason: "No changes to save".to_owned(),
        keyring_state: keyring_state([
            password_keyring_state,
            tls_keyring_state,
            ssh_keyring_state,
        ]),
        validation_errors: warnings.to_vec(),
        plugin_workflows: connection
            .plugin_workflows
            .iter()
            .map(connection_plugin_workflow)
            .collect(),
        ..ConnectionSettingsSnapshot::default()
    }
}

// One store read for all three secret kinds — with the keychain-backed blob
// this is a single IPC instead of three per connection.
fn connection_secrets_from(
    secrets: &[ImportedSecret],
    connection_id: &str,
) -> [(SecretInput, KeyringState); 3] {
    [
        SecretKind::Password,
        SecretKind::SslKeystorePassword,
        SecretKind::AuthPassword,
    ]
    .map(|kind| {
        let value = secrets
            .iter()
            .find(|secret| {
                secret.reference.connection_id == connection_id && secret.reference.kind == kind
            })
            .map(|secret| SecretInput::new(secret.value.expose_secret()))
            .unwrap_or_default();
        (value, KeyringState::Available)
    })
}

fn connection_secrets(connection_id: &str) -> [(SecretInput, KeyringState); 3] {
    let references = [
        SecretKind::Password,
        SecretKind::SslKeystorePassword,
        SecretKind::AuthPassword,
    ]
    .map(|kind| SecretReference {
        connection_id: connection_id.to_owned(),
        kind,
    });
    match default_secret_store().get_all(&references) {
        Ok(values) if values.len() == 3 => {
            let mut values = values.into_iter();
            [(); 3].map(|()| {
                (
                    values
                        .next()
                        .flatten()
                        .map(|secret| SecretInput::new(secret.expose_for_migration()))
                        .unwrap_or_default(),
                    KeyringState::Available,
                )
            })
        }
        _ => [(); 3].map(|()| (SecretInput::default(), KeyringState::Unavailable)),
    }
}

fn keyring_state(states: [KeyringState; 3]) -> KeyringState {
    if states.contains(&KeyringState::Unavailable) {
        KeyringState::Unavailable
    } else if states.contains(&KeyringState::Locked) {
        KeyringState::Locked
    } else {
        KeyringState::Available
    }
}

fn connection_plugin_workflow(config: &ConnectionPluginWorkflowConfig) -> ConnectionPluginWorkflow {
    ConnectionPluginWorkflow {
        plugin_id: config.plugin_id.clone(),
        plugin_name: config.plugin_id.clone(),
        enabled: config.enabled,
        kind: match config.kind {
            StorageConnectionPluginWorkflowKind::Validator => {
                ConnectionPluginWorkflowKind::Validator
            }
            StorageConnectionPluginWorkflowKind::Manipulator => {
                ConnectionPluginWorkflowKind::Manipulator
            }
        },
        direction: match config.direction {
            StorageConnectionPluginDirection::Incoming => ConnectionPluginDirection::Incoming,
            StorageConnectionPluginDirection::Outgoing => ConnectionPluginDirection::Outgoing,
            StorageConnectionPluginDirection::Both => ConnectionPluginDirection::Both,
        },
        topic_filter: config.topic_filter.clone(),
        config: config.config.clone(),
        available: false,
        status: if config.enabled {
            ConnectionPluginWorkflowStatus::MissingPlugin
        } else {
            ConnectionPluginWorkflowStatus::Disabled
        },
        message: String::new(),
    }
}

fn workbench_from_history(history: &ConnectionHistorySnapshot) -> WorkbenchSnapshot {
    let mut workbench = WorkbenchSnapshot::default();
    workbench.publish.topic_history = history.publish_topics.topics.clone();
    workbench.publish.history = history
        .publish_messages
        .messages
        .iter()
        .enumerate()
        .map(|(index, message)| {
            let payload = message.payload.clone().unwrap_or_default().into_bytes();
            let mut badges = Vec::new();
            if message.retained {
                badges.push("retained".to_owned());
            }
            PublishHistoryRow {
                id: (index as u32).saturating_add(1),
                topic: message.topic.clone(),
                timestamp: message
                    .date_time
                    .clone()
                    .unwrap_or_else(|| "migrated".to_owned()),
                qos: message.qos.map(qos).unwrap_or_default(),
                retained: message.retained,
                payload_preview: payload_preview(&payload),
                byte_size: payload.len(),
                payload,
                badges,
                diagnostics: Vec::new(),
            }
        })
        .collect();
    workbench.subscribe = SubscribePaneSnapshot {
        topic_history: history.subscriptions.topics.clone(),
        subscriptions: history
            .subscriptions
            .topics
            .iter()
            .map(|topic| SubscriptionRow {
                topic_filter: topic.clone(),
                qos: QosLevel::Zero,
                message_count: 0,
                active: true,
                messages_visible: true,
                selected: false,
            })
            .collect(),
        ..SubscribePaneSnapshot::default()
    };
    workbench
}

fn global_settings(settings: &Settings) -> GlobalSettingsSnapshot {
    let mut snapshot = GlobalSettingsSnapshot {
        language: settings
            .saved_locale
            .clone()
            .or_else(|| settings.current_locale.clone())
            .unwrap_or_else(|| "system".to_owned()),
        keyring_backend: normalize_keyring_backend(
            settings.keyring_identifier.as_deref().unwrap_or("os"),
        ),
        update_checks_enabled: settings.search_updates,
        last_update_check: "Not checked this session".to_owned(),
        cleanup_status: "Sensitive values remain outside the UI snapshot".to_owned(),
        search_use_regex: settings.use_regex_for_search,
        search_ignore_case: settings.use_ignore_case,
        reduce_motion: settings.reduce_motion,
        use_default_plugin_repository: settings.use_default_repo,
        install_bundled_plugins: settings.install_bundled_plugins,
        bundled_plugins_url: settings.bundled_plugins_url.clone().unwrap_or_default(),
        plugin_repositories: settings
            .plugin_repositories
            .iter()
            .map(|(id, url)| PluginRepositoryRow {
                id: id.clone(),
                url: url.clone(),
            })
            .collect(),
        plugin_states: settings
            .plugin_states
            .iter()
            .map(|(plugin_id, state)| {
                (
                    plugin_id.clone(),
                    PluginStateSnapshot {
                        enabled: state.enabled,
                    },
                )
            })
            .collect(),
        plugin_hooks: settings
            .plugin_hooks
            .iter()
            .map(|(plugin_id, hooks)| {
                (
                    plugin_id.clone(),
                    hooks.iter().map(plugin_hook_settings).collect(),
                )
            })
            .collect(),
        first_start: settings.first_start,
        config_version: settings
            .config_created_with_correo_version
            .clone()
            .unwrap_or_else(|| "unknown".to_owned()),
        ..GlobalSettingsSnapshot::default()
    };
    if let Some(geometry) = &settings.global_ui_settings {
        snapshot.window_geometry = format!(
            "{:.0}x{:.0} at {:.0},{:.0}",
            geometry.window_width,
            geometry.window_height,
            geometry.window_position_x,
            geometry.window_position_y
        );
    }
    snapshot
}

fn plugin_hook_settings(settings: &PluginHookSettings) -> PluginHookSettingsSnapshot {
    PluginHookSettingsSnapshot {
        hook: plugin_hook_kind(settings.hook),
        enabled: settings.enabled,
        target: settings.target.clone(),
        config_json: settings.config_json.clone(),
    }
}

fn plugin_hook_kind(kind: StoragePluginHookKind) -> crate::PluginHookKind {
    match kind {
        StoragePluginHookKind::IncomingTransform => crate::PluginHookKind::IncomingTransform,
        StoragePluginHookKind::OutgoingTransform => crate::PluginHookKind::OutgoingTransform,
        StoragePluginHookKind::Validator => crate::PluginHookKind::Validator,
        StoragePluginHookKind::DetailTransform => crate::PluginHookKind::DetailTransform,
        StoragePluginHookKind::DetailFormatter => crate::PluginHookKind::DetailFormatter,
    }
}

fn badges(connection: &ConnectionConfig) -> Vec<ConnectionBadge> {
    let mut badges = Vec::new();
    if connection
        .username
        .as_ref()
        .is_some_and(|name| !name.trim().is_empty())
    {
        badges.push(ConnectionBadge::Credentials);
    }
    if connection.ssl != TlsSsl::Off {
        badges.push(ConnectionBadge::Tls);
    }
    if connection.proxy != Proxy::Off {
        badges.push(ConnectionBadge::Proxy);
    }
    if connection.lwt == Lwt::On {
        badges.push(ConnectionBadge::Lwt);
    }
    badges
}

fn theme_mode(settings: Option<&ThemeSettings>) -> Option<ThemeMode> {
    settings
        .and_then(|settings| settings.active_theme.as_ref())
        .and_then(|theme| theme.name.as_deref())
        .map(ThemeMode::parse)
}

fn normalize_publish_history_ids(workbench: &mut WorkbenchSnapshot) {
    let mut next_id = 1u32;
    for row in &mut workbench.publish.history {
        if row.id == 0 {
            row.id = next_id;
        }
        next_id = next_id.max(row.id.saturating_add(1));
    }
    if workbench.publish.selected_history_id == Some(0) {
        workbench.publish.selected_history_id = None;
    }
}

fn payload_preview(payload: &[u8]) -> String {
    const LIMIT: usize = 96;
    let mut preview = String::from_utf8_lossy(payload).replace(['\n', '\r'], " ");
    if preview.len() > LIMIT {
        let truncated = preview.chars().take(LIMIT).collect::<String>();
        preview = format!("{truncated}...");
    }
    preview
}

fn mqtt_label(version: MqttVersion) -> &'static str {
    match version {
        MqttVersion::Mqtt311 => "MQTT 3.1.1",
        MqttVersion::Mqtt50 => "MQTT v5",
    }
}

fn qos(qos: StorageQos) -> QosLevel {
    match qos {
        StorageQos::AtMostOnce => QosLevel::Zero,
        StorageQos::AtLeastOnce => QosLevel::One,
        StorageQos::ExactlyOnce => QosLevel::Two,
    }
}

fn history_activity(history: &ConnectionHistorySnapshot) -> String {
    format!(
        "{} publish topic(s), {} subscription(s) migrated",
        history.publish_topics.topics.len(),
        history.subscriptions.topics.len()
    )
}

fn auth_label(connection: &ConnectionConfig) -> &'static str {
    match connection.auth {
        Auth::Off => "No Auth",
        Auth::Password => "Password",
        Auth::Keyfile => "Keyfile",
    }
}

fn password_status(connection: &ConnectionConfig, password: &SecretInput) -> &'static str {
    if !password.is_empty() {
        "MQTT password managed by keyring"
    } else if connection.username.is_some() {
        "MQTT password missing from keyring"
    } else {
        "No MQTT password configured"
    }
}

fn tls_label(connection: &ConnectionConfig) -> &'static str {
    match connection.ssl {
        TlsSsl::Off => "No TLS/SSL",
        TlsSsl::Keystore => "Keystore",
    }
}

fn tls_password_status(connection: &ConnectionConfig, password: &SecretInput) -> &'static str {
    if !password.is_empty() {
        "SSL password managed by keyring"
    } else if connection.ssl_keystore.is_some() {
        "SSL password missing from keyring"
    } else {
        "No SSL password configured"
    }
}

fn proxy_label(connection: &ConnectionConfig) -> &'static str {
    match connection.proxy {
        Proxy::Off => "No proxy/tunnel",
        Proxy::Ssh => "SSH",
    }
}

fn ssh_password_status(connection: &ConnectionConfig, password: &SecretInput) -> &'static str {
    if !password.is_empty() {
        return match connection.auth {
            Auth::Keyfile => "SSH key passphrase managed by keyring",
            _ => "SSH password managed by keyring",
        };
    }

    match connection.auth {
        Auth::Password => "SSH password missing from keyring",
        Auth::Keyfile => "No SSH key passphrase configured",
        Auth::Off => "No SSH password configured",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PluginLoadState, PluginSource, PluginStatus};

    const TEST_REPOSITORY_JSON: &str =
        include_str!("../../correo-plugins/tests/fixtures/repository.json");

    fn bundled_plugin_ids() -> Vec<String> {
        [
            "org.correomqtt.plugins.advanced-validator",
            "org.correomqtt.plugins.base64",
            "org.correomqtt.plugins.contains-string-validator",
            "org.correomqtt.plugins.json-format",
            "org.correomqtt.plugins.save-manipulator",
            "org.correomqtt.plugins.system-topic",
            "org.correomqtt.plugins.xml-format",
            "org.correomqtt.plugins.xml-xsd-validator",
            "org.correomqtt.plugins.zip-manipulator",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }

    #[test]
    fn current_startup_populates_marketplace_and_installs_bundled_plugins() {
        let state = startup_state_from_current_with_plugins(
            AppConfig::default(),
            HistoryPersistenceSnapshot::default(),
            BTreeMap::new(),
            ScriptPersistenceSnapshot::default(),
            Vec::new(),
            ThemeMode::System,
            vec![TEST_REPOSITORY_JSON.to_owned()],
            bundled_plugin_ids(),
            Vec::new(),
            Vec::new(),
        );
        let plugins = &state.snapshot.plugins;

        assert_eq!(plugins.load_state, PluginLoadState::Ready);
        assert_eq!(plugins.marketplace_plugins.len(), 9);
        assert_eq!(plugins.plugins.len(), 9);
        assert!(!plugins.selected_plugin_id.is_empty());
        assert!(!plugins.selected_marketplace_plugin_id.is_empty());
        assert!(plugins.plugins.iter().all(|plugin| {
            plugin.source == PluginSource::Bundled && plugin.status == PluginStatus::Active
        }));
        assert!(plugins
            .marketplace_plugins
            .iter()
            .filter(|plugin| plugin.install_source.is_bundled())
            .all(|plugin| plugin.installed_plugin_id.as_deref() == Some(plugin.id.as_str())));
        assert_eq!(
            plugins
                .marketplace_plugins
                .iter()
                .find(|plugin| plugin.id == "org.correomqtt.plugins.save-manipulator")
                .and_then(|plugin| plugin.installed_plugin_id.as_deref()),
            Some("org.correomqtt.plugins.save-manipulator")
        );
    }

    #[test]
    fn bundled_plugin_ids_keep_file_installed_plugins_non_uninstallable() {
        let repository_json =
            repository_json_with_local_package_source("org.correomqtt.plugins.advanced-validator");
        let state = startup_state_from_current_with_plugins(
            AppConfig::default(),
            HistoryPersistenceSnapshot::default(),
            BTreeMap::new(),
            ScriptPersistenceSnapshot::default(),
            Vec::new(),
            ThemeMode::System,
            vec![repository_json],
            vec!["org.correomqtt.plugins.advanced-validator".to_owned()],
            Vec::new(),
            Vec::new(),
        );
        let plugin = state
            .snapshot
            .plugins
            .plugins
            .iter()
            .find(|plugin| plugin.id == "org.correomqtt.plugins.advanced-validator")
            .expect("bundled plugin should be installed");

        assert_eq!(plugin.source, PluginSource::Bundled);
        assert!(!plugin.can_uninstall());
    }

    #[test]
    fn current_startup_keeps_bundled_plugins_uninstalled_when_setting_is_off() {
        let mut config = AppConfig::default();
        config.settings.install_bundled_plugins = false;

        let state = startup_state_from_current_with_plugins(
            config,
            HistoryPersistenceSnapshot::default(),
            BTreeMap::new(),
            ScriptPersistenceSnapshot::default(),
            Vec::new(),
            ThemeMode::System,
            vec![TEST_REPOSITORY_JSON.to_owned()],
            bundled_plugin_ids(),
            Vec::new(),
            Vec::new(),
        );
        let plugins = &state.snapshot.plugins;

        assert_eq!(plugins.load_state, PluginLoadState::Ready);
        assert_eq!(plugins.marketplace_plugins.len(), 9);
        assert!(plugins.plugins.is_empty());
        assert!(plugins
            .marketplace_plugins
            .iter()
            .all(|plugin| plugin.installed_plugin_id.is_none()));
    }

    fn repository_json_with_local_package_source(plugin_id: &str) -> String {
        let mut value = serde_json::from_str::<serde_json::Value>(TEST_REPOSITORY_JSON).unwrap();
        let plugins = value
            .get_mut("plugins")
            .and_then(serde_json::Value::as_array_mut)
            .unwrap();
        let plugin = plugins
            .iter_mut()
            .find(|plugin| {
                plugin
                    .pointer("/manifest/id")
                    .and_then(serde_json::Value::as_str)
                    == Some(plugin_id)
            })
            .unwrap();
        plugin["install_source"] = serde_json::json!({
            "kind": "local_package",
            "path": format!("plugins/{plugin_id}"),
        });
        serde_json::to_string(&value).unwrap()
    }

    #[test]
    fn connection_secrets_from_hydrates_matching_imported_values() {
        use correo_storage::current::SecretMaterial;
        let secret = |connection_id: &str, kind, value: &str| ImportedSecret {
            reference: SecretReference {
                connection_id: connection_id.to_owned(),
                kind,
            },
            value: SecretMaterial::new(value),
        };
        let secrets = vec![
            secret("c1", SecretKind::Password, "mqtt-pw"),
            secret("c1", SecretKind::AuthPassword, "ssh-pw"),
            // A secret for another connection must be ignored.
            secret("other", SecretKind::Password, "nope"),
        ];

        let [(password, _), (tls, _), (ssh, _)] = connection_secrets_from(&secrets, "c1");
        assert_eq!(password.expose_for_ui(), "mqtt-pw");
        assert_eq!(tls.expose_for_ui(), "");
        assert_eq!(ssh.expose_for_ui(), "ssh-pw");
    }
}
