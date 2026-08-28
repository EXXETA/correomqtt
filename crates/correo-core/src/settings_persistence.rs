use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use correo_storage::current::{
    default_secret_store, Auth as StorageAuth, BuiltInBrokerConfig, ConfigStore,
    ConnectionConfig as StorageConnectionConfig,
    ConnectionPluginDirection as StorageConnectionPluginDirection, ConnectionPluginWorkflowConfig,
    ConnectionPluginWorkflowKind as StorageConnectionPluginWorkflowKind, ImportedSecret,
    Lwt as StorageLwt, MqttVersion as StorageMqttVersion, PluginHookKind as StoragePluginHookKind,
    PluginHookSettings, PluginStateSettings, Protocol as StorageProtocol, Proxy as StorageProxy,
    Qos as StorageQos, SecretKind, SecretMaterial, SecretReference, SecretStore, Settings,
    TlsSsl as StorageTlsSsl,
};
use thiserror::Error;

use crate::{
    normalize_keyring_backend, ConnectionPluginDirection, ConnectionPluginWorkflow,
    ConnectionPluginWorkflowKind, ConnectionSettingsSnapshot, GlobalSettingsSnapshot,
    PluginHookKind, PluginHookSettingsSnapshot, PluginRepositoryRow, QosLevel, ThemeMode,
};

#[derive(Clone, PartialEq, Eq)]
pub struct BuiltInBrokerPersistenceSnapshot {
    pub port: String,
    pub credentials_enabled: bool,
    pub username: String,
    pub password: String,
}

impl std::fmt::Debug for BuiltInBrokerPersistenceSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltInBrokerPersistenceSnapshot")
            .field("port", &self.port)
            .field("credentials_enabled", &self.credentials_enabled)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsPersistenceCommand {
    Save {
        theme_mode: ThemeMode,
        settings: Box<GlobalSettingsSnapshot>,
    },
    SaveConnectionPluginWorkflows {
        connection_id: String,
        workflows: Vec<ConnectionPluginWorkflow>,
    },
    SaveConnectionSettings {
        connection_id: String,
        settings: Box<ConnectionSettingsSnapshot>,
    },
    SaveImportedConnections {
        connections: Vec<StorageConnectionConfig>,
        secrets: Vec<ImportedSecret>,
    },
    DeleteConnection {
        connection_id: String,
    },
    SaveConnectionOrder {
        connection_ids: Vec<String>,
    },
    SaveBuiltInBroker {
        broker: BuiltInBrokerPersistenceSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceEvent {
    Saved,
    Failed { error: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SettingsDispatchError {
    #[error("settings persistence worker is stopped")]
    Stopped,
}

#[derive(Debug)]
pub struct SettingsPersistenceWorker {
    sender: Option<Sender<SettingsPersistenceCommand>>,
    events: Receiver<SettingsPersistenceEvent>,
    handle: Option<JoinHandle<()>>,
}

impl SettingsPersistenceWorker {
    pub fn start(root: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let store = ConfigStore::new(root.into());

        let handle = thread::spawn(move || {
            while let Ok(command) = receiver.recv() {
                let event = apply_settings_command(&store, command);
                let _ = events_sender.send(event);
            }
        });

        Self {
            sender: Some(sender),
            events,
            handle: Some(handle),
        }
    }

    pub fn dispatch(
        &self,
        command: SettingsPersistenceCommand,
    ) -> Result<(), SettingsDispatchError> {
        self.sender
            .as_ref()
            .ok_or(SettingsDispatchError::Stopped)?
            .send(command)
            .map_err(|_| SettingsDispatchError::Stopped)
    }

    pub fn try_recv_event(&self) -> Option<SettingsPersistenceEvent> {
        self.events.try_recv().ok()
    }

    pub fn recv_event_timeout(&self, timeout: Duration) -> Option<SettingsPersistenceEvent> {
        self.events.recv_timeout(timeout).ok()
    }
}

impl Drop for SettingsPersistenceWorker {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn apply_settings_command(
    store: &ConfigStore,
    command: SettingsPersistenceCommand,
) -> SettingsPersistenceEvent {
    let result = match command {
        SettingsPersistenceCommand::Save {
            theme_mode,
            settings,
        } => store.save_global_settings(theme_name(&theme_mode), storage_settings(*settings)),
        SettingsPersistenceCommand::SaveConnectionPluginWorkflows {
            connection_id,
            workflows,
        } => store.save_connection_plugin_workflows(
            &connection_id,
            workflows.into_iter().map(storage_plugin_workflow).collect(),
        ),
        SettingsPersistenceCommand::SaveConnectionSettings {
            connection_id,
            settings,
        } => save_connection_settings(
            store,
            default_secret_store().as_ref(),
            connection_id,
            *settings,
        ),
        SettingsPersistenceCommand::SaveImportedConnections {
            connections,
            secrets,
        } => {
            save_imported_connections(store, default_secret_store().as_ref(), connections, secrets)
        }
        SettingsPersistenceCommand::DeleteConnection { connection_id } => {
            delete_connection(store, default_secret_store().as_ref(), connection_id)
        }
        SettingsPersistenceCommand::SaveConnectionOrder { connection_ids } => {
            store.save_connection_order(&connection_ids)
        }
        SettingsPersistenceCommand::SaveBuiltInBroker { broker } => {
            save_built_in_broker(store, default_secret_store().as_ref(), broker)
        }
    };

    match result {
        Ok(_) => SettingsPersistenceEvent::Saved,
        Err(error) => SettingsPersistenceEvent::Failed {
            error: error.to_string(),
        },
    }
}

fn save_imported_connections(
    store: &ConfigStore,
    secret_store: &dyn SecretStore,
    connections: Vec<StorageConnectionConfig>,
    secrets: Vec<ImportedSecret>,
) -> correo_storage::Result<correo_storage::current::AppConfig> {
    apply_secrets_then_save(secret_store, &secrets, &[], || {
        store.save_connections(connections)
    })
}

// Writes secrets before the config so a failed secret write never persists a
// connection without its credentials, and snapshots the previous secret state
// so a failed config write can restore the prior values.
fn apply_secrets_then_save(
    secret_store: &dyn SecretStore,
    puts: &[ImportedSecret],
    deletes: &[SecretReference],
    save_config: impl FnOnce() -> correo_storage::Result<correo_storage::current::AppConfig>,
) -> correo_storage::Result<correo_storage::current::AppConfig> {
    let affected: Vec<SecretReference> = puts
        .iter()
        .map(|secret| secret.reference.clone())
        .chain(deletes.iter().cloned())
        .collect();
    let previous = secret_store.get_all(&affected)?;

    secret_store.apply(puts, deletes)?;

    match save_config() {
        Ok(config) => Ok(config),
        Err(config_error) => {
            restore_secrets(secret_store, &affected, previous)?;
            Err(config_error)
        }
    }
}

fn restore_secrets(
    secret_store: &dyn SecretStore,
    affected: &[SecretReference],
    previous: Vec<Option<SecretMaterial>>,
) -> correo_storage::Result<()> {
    let mut puts = Vec::new();
    let mut deletes = Vec::new();
    for (reference, value) in affected.iter().zip(previous) {
        match value {
            Some(value) => puts.push(ImportedSecret {
                reference: reference.clone(),
                value,
            }),
            None => deletes.push(reference.clone()),
        }
    }
    secret_store.apply(&puts, &deletes)
}

fn delete_connection(
    store: &ConfigStore,
    secret_store: &dyn SecretStore,
    connection_id: String,
) -> correo_storage::Result<correo_storage::current::AppConfig> {
    // Config first for deletes: removing the connection record before its
    // secrets means a failed secret cleanup only leaves harmless orphans.
    let config = store.delete_connection(&connection_id)?;
    let deletes = [
        secret_reference(&connection_id, SecretKind::Password),
        secret_reference(&connection_id, SecretKind::SslKeystorePassword),
        secret_reference(&connection_id, SecretKind::AuthPassword),
    ];
    secret_store.apply(&[], &deletes)?;
    Ok(config)
}

fn secret_reference(connection_id: &str, kind: SecretKind) -> SecretReference {
    SecretReference {
        connection_id: connection_id.to_owned(),
        kind,
    }
}

fn save_connection_settings(
    store: &ConfigStore,
    secret_store: &dyn SecretStore,
    connection_id: String,
    settings: ConnectionSettingsSnapshot,
) -> correo_storage::Result<correo_storage::current::AppConfig> {
    let connection = storage_connection(connection_id.clone(), settings.clone());
    let mut puts = Vec::new();
    let mut deletes = Vec::new();
    for (kind, value, current_status) in [
        (
            SecretKind::Password,
            &settings.password,
            settings.password_status.as_str(),
        ),
        (
            SecretKind::SslKeystorePassword,
            &settings.tls_keystore_password,
            settings.tls_password_status.as_str(),
        ),
        (
            SecretKind::AuthPassword,
            &settings.ssh_password,
            settings.ssh_password_status.as_str(),
        ),
    ] {
        let reference = secret_reference(&connection_id, kind);
        if value.is_empty() {
            if secret_was_configured(current_status) {
                deletes.push(reference);
            }
        } else {
            puts.push(ImportedSecret {
                reference,
                value: SecretMaterial::new(value.expose_for_ui()),
            });
        }
    }
    apply_secrets_then_save(secret_store, &puts, &deletes, || {
        store.save_connection(connection)
    })
}

fn secret_was_configured(status: &str) -> bool {
    status.contains("managed by keyring") || status.contains("missing from keyring")
}

pub fn migrate_legacy_built_in_broker(
    store: &ConfigStore,
    secret_store: &dyn SecretStore,
) -> correo_storage::Result<correo_storage::current::AppConfig> {
    let config = store.load()?;
    if config.built_in_broker.password.is_empty() {
        return Ok(config);
    }
    let reference = store.built_in_broker_secret_reference(&config)?;
    let replacement = BuiltInBrokerConfig {
        port: config.built_in_broker.port.clone(),
        credentials_enabled: config.built_in_broker.credentials_enabled,
        username: config.built_in_broker.username.clone(),
        password: String::new(),
    };
    let secret = ImportedSecret {
        reference: reference.clone(),
        value: SecretMaterial::new(config.built_in_broker.password.clone()),
    };
    apply_secrets_then_save(secret_store, std::slice::from_ref(&secret), &[], || {
        store.save_built_in_broker_with_secret_reference(replacement, reference)
    })
}

fn save_built_in_broker(
    store: &ConfigStore,
    secret_store: &dyn SecretStore,
    snapshot: BuiltInBrokerPersistenceSnapshot,
) -> correo_storage::Result<correo_storage::current::AppConfig> {
    let config = store.load_or_default()?;
    let reference = store.built_in_broker_secret_reference(&config)?;
    let broker = BuiltInBrokerConfig {
        port: snapshot.port,
        credentials_enabled: snapshot.credentials_enabled,
        username: snapshot.username,
        password: String::new(),
    };
    let mut puts = Vec::new();
    let mut deletes = Vec::new();
    if snapshot.credentials_enabled && !snapshot.password.is_empty() {
        puts.push(ImportedSecret {
            reference: reference.clone(),
            value: SecretMaterial::new(snapshot.password),
        });
    } else if !snapshot.credentials_enabled {
        deletes.push(reference.clone());
    }
    apply_secrets_then_save(secret_store, &puts, &deletes, || {
        store.save_built_in_broker_with_secret_reference(broker, reference)
    })
}

pub(crate) fn storage_connection(
    connection_id: String,
    settings: ConnectionSettingsSnapshot,
) -> StorageConnectionConfig {
    StorageConnectionConfig {
        id: connection_id,
        name: settings.profile_name.trim().to_owned(),
        // MQTT is the only protocol today; a future protocol selector would
        // set this from the UI settings.
        protocol: StorageProtocol::Mqtt,
        url: settings.host.trim().to_owned(),
        port: parse_port(&settings.port, 1883),
        client_id: non_empty(settings.client_id),
        username: non_empty(settings.username),
        clean_session: settings.clean_session,
        mqtt_version: storage_mqtt_version(&settings.mqtt_version),
        ssl: storage_tls(&settings.tls_mode),
        ssl_keystore: non_empty(settings.tls_store),
        ssl_host_verification: settings.tls_host_verification,
        proxy: storage_proxy(&settings.proxy_mode),
        ssh_host: non_empty(settings.ssh_host),
        ssh_port: parse_port(&settings.ssh_port, 22),
        local_port: parse_optional_port(&settings.local_mqtt_port),
        auth: storage_auth(&settings.auth_mode),
        auth_username: non_empty(settings.auth_username),
        auth_keyfile: non_empty(settings.ssh_key_file),
        lwt: if settings.lwt_enabled {
            StorageLwt::On
        } else {
            StorageLwt::Off
        },
        lwt_topic: non_empty(settings.lwt_topic),
        lwt_qos: settings
            .lwt_enabled
            .then_some(storage_qos(settings.lwt_qos)),
        lwt_retained: settings.lwt_retained,
        lwt_payload: non_empty(settings.lwt_payload),
        connection_ui_settings: None,
        publish_list_view_config: None,
        subscribe_list_view_config: None,
        plugin_workflows: settings
            .plugin_workflows
            .into_iter()
            .map(storage_plugin_workflow)
            .collect(),
    }
}

fn storage_mqtt_version(value: &str) -> StorageMqttVersion {
    if value == "MQTT 3.1.1" {
        StorageMqttVersion::Mqtt311
    } else {
        StorageMqttVersion::Mqtt50
    }
}

fn storage_tls(value: &str) -> StorageTlsSsl {
    if value == "Keystore" {
        StorageTlsSsl::Keystore
    } else {
        StorageTlsSsl::Off
    }
}

fn storage_proxy(value: &str) -> StorageProxy {
    if value == "SSH" {
        StorageProxy::Ssh
    } else {
        StorageProxy::Off
    }
}

fn storage_auth(value: &str) -> StorageAuth {
    match value {
        "Password" => StorageAuth::Password,
        "Keyfile" => StorageAuth::Keyfile,
        _ => StorageAuth::Off,
    }
}

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
