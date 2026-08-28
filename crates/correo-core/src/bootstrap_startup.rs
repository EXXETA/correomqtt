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
