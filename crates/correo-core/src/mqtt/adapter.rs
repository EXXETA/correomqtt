use std::collections::HashMap;

use correo_mqtt::{
    ConnectionId, LastWill, MqttAuth, MqttConnectionOptions, MqttEndpoint, MqttError,
    MqttProtocolVersion, PublishRequest, Qos, SecretBytes, SecretString, SshAuth, SshHostKeyPolicy,
    SshTunnelOptions, Subscription, TlsConfig, TlsHostVerification, TlsOptions, TlsTrustRoots,
    UnsubscribeRequest,
};
use thiserror::Error;

use crate::{
    AppCommand, AppSnapshot, ConnectDisabledReason, ConnectionSettingsSnapshot, ConnectionState,
    ConnectionSummary, MqttCommand, MqttOperation, QosLevel,
};

pub(crate) fn commands_for_app_command(
    command: &AppCommand,
    snapshot: &AppSnapshot,
    connection_settings: &HashMap<ConnectionId, ConnectionSettingsSnapshot>,
) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
    match command {
        AppCommand::Connect(connection_id) => {
            connect_command(*connection_id, false, snapshot, connection_settings)
        }
        AppCommand::Reconnect(connection_id) => {
            connect_command(*connection_id, true, snapshot, connection_settings)
        }
        AppCommand::Disconnect(connection_id) => Ok(vec![MqttCommand::Disconnect {
            connection_id: *connection_id,
        }]),
        AppCommand::Publish => publish_command(snapshot),
        AppCommand::Subscribe => subscribe_command(snapshot),
        AppCommand::Unsubscribe(topic_filter) => unsubscribe_command(topic_filter, snapshot),
        AppCommand::UnsubscribeAll => unsubscribe_all_commands(snapshot),
        AppCommand::RunScript => script_connection_connect_command(snapshot, connection_settings),
        AppCommand::CancelUnsubscribeAll | AppCommand::ConfirmUnsubscribeAll => Ok(Vec::new()),
        AppCommand::Mqtt(command) => Ok(vec![command.clone()]),
        _ => Ok(Vec::new()),
    }
}

fn script_connection_connect_command(
    snapshot: &AppSnapshot,
    connection_settings: &HashMap<ConnectionId, ConnectionSettingsSnapshot>,
) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
    let Some(connection_id) = snapshot
        .scripts
        .selected_connection_id
        .as_deref()
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .map(ConnectionId::from_uuid)
    else {
        return Ok(Vec::new());
    };
    connect_command(connection_id, false, snapshot, connection_settings)
}

#[derive(Debug, Error)]
pub enum MqttCommandBuildError {
    #[error("connection is not known: {connection_id}")]
    UnknownConnection { connection_id: ConnectionId },
    #[error("connection settings are unavailable for {connection_id}")]
    MissingConnectionSettings { connection_id: ConnectionId },
    #[error("no connection is selected for MQTT {operation}")]
    NoSelectedConnection { operation: MqttOperation },
    #[error("MQTT {operation} requires a connected broker for {connection_id}")]
    NoOpenConnection {
        operation: MqttOperation,
        connection_id: ConnectionId,
    },
    #[error("connection {connection_id} has an invalid MQTT port")]
    InvalidPort { connection_id: ConnectionId },
    #[error("MQTT {operation} request is invalid: {source}")]
    InvalidRequest {
        operation: MqttOperation,
        #[source]
        source: MqttError,
    },
}

fn connect_command(
    connection_id: ConnectionId,
    reconnect: bool,
    snapshot: &AppSnapshot,
    connection_settings: &HashMap<ConnectionId, ConnectionSettingsSnapshot>,
) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
    let Some(connection) = connection(snapshot, connection_id) else {
        return Err(MqttCommandBuildError::UnknownConnection { connection_id });
    };

    if !reconnect && !connection.can_connect() {
        return Ok(Vec::new());
    }
    if reconnect
        && matches!(
            connection.disabled_reason,
            Some(ConnectDisabledReason::MissingHost)
        )
    {
        return Ok(Vec::new());
    }

    let options = connection_options(connection_id, snapshot, connection_settings)?;
    let command = if reconnect {
        MqttCommand::Reconnect { options }
    } else {
        MqttCommand::Connect { options }
    };
    Ok(vec![command])
}

fn publish_command(snapshot: &AppSnapshot) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
    let connection_id = connected_connection_id(snapshot, MqttOperation::Publish)?;
    let publish = &snapshot.workbench.publish;
    let topic = publish.topic.trim();
    if topic.is_empty() {
        return Ok(Vec::new());
    }

    let request = PublishRequest::new(
        topic,
        publish.payload.as_bytes().to_vec(),
        qos(publish.qos),
        publish.retained,
    )
    .map_err(|source| MqttCommandBuildError::InvalidRequest {
        operation: MqttOperation::Publish,
        source,
    })?;

    Ok(vec![MqttCommand::Publish {
        connection_id,
        request,
        diagnostics: Vec::new(),
    }])
}

fn subscribe_command(snapshot: &AppSnapshot) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
    let connection_id = connected_connection_id(snapshot, MqttOperation::Subscribe)?;
    let topic_filter = snapshot.workbench.subscribe.topic.trim();
    if topic_filter.is_empty() {
        return Ok(Vec::new());
    }

    let subscription = Subscription::new(topic_filter, qos(snapshot.workbench.subscribe.qos))
        .map_err(|source| MqttCommandBuildError::InvalidRequest {
            operation: MqttOperation::Subscribe,
            source,
        })?;

    Ok(vec![MqttCommand::Subscribe {
        connection_id,
        subscription,
    }])
}

fn unsubscribe_command(
    topic_filter: &str,
    snapshot: &AppSnapshot,
) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
    let connection_id = connected_connection_id(snapshot, MqttOperation::Unsubscribe)?;
    let topic_filter = topic_filter.trim();
    if topic_filter.is_empty() {
        return Ok(Vec::new());
    }

    let request = UnsubscribeRequest::new(topic_filter).map_err(|source| {
        MqttCommandBuildError::InvalidRequest {
            operation: MqttOperation::Unsubscribe,
            source,
        }
    })?;

    Ok(vec![MqttCommand::Unsubscribe {
        connection_id,
        request,
    }])
}

fn unsubscribe_all_commands(
    snapshot: &AppSnapshot,
) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
    let connection_id = connected_connection_id(snapshot, MqttOperation::Unsubscribe)?;
    snapshot
        .workbench
        .subscribe
        .subscriptions
        .iter()
        .filter(|subscription| subscription.active)
        .map(|subscription| {
            let request =
                UnsubscribeRequest::new(subscription.topic_filter.as_str()).map_err(|source| {
                    MqttCommandBuildError::InvalidRequest {
                        operation: MqttOperation::Unsubscribe,
                        source,
                    }
                })?;
            Ok(MqttCommand::Unsubscribe {
                connection_id,
                request,
            })
        })
        .collect()
}

fn connection_options(
    connection_id: ConnectionId,
    snapshot: &AppSnapshot,
    connection_settings: &HashMap<ConnectionId, ConnectionSettingsSnapshot>,
) -> Result<MqttConnectionOptions, MqttCommandBuildError> {
    let connection = connection(snapshot, connection_id)
        .ok_or(MqttCommandBuildError::UnknownConnection { connection_id })?;
    let settings = settings_for(connection_id, snapshot, connection_settings);

    let name = settings
        .map(|settings| settings.profile_name.trim())
        .filter(|name| !name.is_empty())
        .unwrap_or(&connection.name);
    let (host, port) = endpoint_parts(connection_id, connection, settings)?;
    let endpoint =
        MqttEndpoint::new(host, port).map_err(|source| MqttCommandBuildError::InvalidRequest {
            operation: MqttOperation::Connect,
            source,
        })?;
    let mut options = MqttConnectionOptions::new(connection_id, name, endpoint);
    if let Some(settings) = settings {
        options.client_id = non_empty(settings.client_id.trim());
        options.protocol_version = MqttProtocolVersion::try_from(settings.mqtt_version.as_str())
            .map_err(|source| MqttCommandBuildError::InvalidRequest {
                operation: MqttOperation::Connect,
                source,
            })?;
        options.clean_start = settings.clean_session;
        if !settings.username.trim().is_empty() || !settings.password.is_empty() {
            options.auth = MqttAuth::UsernamePassword {
                username: non_empty(settings.username.trim()),
                password: SecretString::new(settings.password.expose_for_ui().to_owned()),
            };
        }
        if settings.tls_mode != "No TLS/SSL" {
            options.tls = TlsConfig::Enabled(tls_options(settings));
        }
        options.ssh_tunnel = ssh_tunnel_options(connection_id, settings)?;
        options.last_will = last_will(settings)?;
    } else {
        options.protocol_version = MqttProtocolVersion::try_from(connection.mqtt_version.as_str())
            .map_err(|source| MqttCommandBuildError::InvalidRequest {
                operation: MqttOperation::Connect,
                source,
            })?;
    }
    Ok(options)
}

fn tls_options(settings: &ConnectionSettingsSnapshot) -> TlsOptions {
    let mut tls = TlsOptions::default();
    if !settings.tls_host_verification {
        tls.host_verification = TlsHostVerification::DisabledInsecure;
    }

    let store = settings.tls_store.trim();
    if store.is_empty() {
        return tls;
    }

    let path = store.to_owned();
    if is_pkcs12_path(store) {
        tls.trust_roots = TlsTrustRoots::Pkcs12 {
            path: Some(path),
            der: SecretBytes::new(Vec::new()),
            password: settings
                .tls_keystore_password
                .expose_non_empty()
                .map(|value| SecretString::new(value.to_owned())),
        };
    } else {
        tls.trust_roots = TlsTrustRoots::PemBundle {
            path: Some(path),
            pem: None,
        };
    }
    tls
}

fn is_pkcs12_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".p12") || lower.ends_with(".pfx")
}

fn settings_for<'a>(
    connection_id: ConnectionId,
    snapshot: &'a AppSnapshot,
    connection_settings: &'a HashMap<ConnectionId, ConnectionSettingsSnapshot>,
) -> Option<&'a ConnectionSettingsSnapshot> {
    connection_settings.get(&connection_id).or_else(|| {
        (snapshot.selected_connection == Some(connection_id))
            .then_some(&snapshot.connection_settings)
    })
}

fn endpoint_parts(
    connection_id: ConnectionId,
    connection: &ConnectionSummary,
    settings: Option<&ConnectionSettingsSnapshot>,
) -> Result<(String, u16), MqttCommandBuildError> {
    if let Some(settings) = settings {
        let port = settings
            .port
            .trim()
            .parse::<u16>()
            .map_err(|_| MqttCommandBuildError::InvalidPort { connection_id })?;
        return Ok((settings.host.trim().to_owned(), port));
    }

    let Some((host, port)) = connection.endpoint.rsplit_once(':') else {
        return Err(MqttCommandBuildError::MissingConnectionSettings { connection_id });
    };
    let port = port
        .trim()
        .parse::<u16>()
        .map_err(|_| MqttCommandBuildError::InvalidPort { connection_id })?;
    Ok((host.trim().to_owned(), port))
}

fn last_will(
    settings: &ConnectionSettingsSnapshot,
) -> Result<Option<LastWill>, MqttCommandBuildError> {
    if !settings.lwt_enabled || settings.lwt_topic.trim().is_empty() {
        return Ok(None);
    }
    let will = LastWill {
        topic: settings.lwt_topic.trim().try_into().map_err(|source| {
            MqttCommandBuildError::InvalidRequest {
                operation: MqttOperation::Connect,
                source,
            }
        })?,
        payload: settings.lwt_payload.as_bytes().to_vec(),
        qos: Qos::AtLeastOnce,
        retain: settings.lwt_retained,
    };
    Ok(Some(will))
}

fn ssh_tunnel_options(
    connection_id: ConnectionId,
    settings: &ConnectionSettingsSnapshot,
) -> Result<Option<SshTunnelOptions>, MqttCommandBuildError> {
    if settings.proxy_mode != "SSH"
        || settings.ssh_host.trim().is_empty()
        || settings.auth_username.trim().is_empty()
        || settings.auth_mode == "No Auth"
    {
        return Ok(None);
    }
    let port = settings
        .ssh_port
        .trim()
        .parse::<u16>()
        .map_err(|_| MqttCommandBuildError::InvalidPort { connection_id })?;
    let local_bind_port = optional_port(connection_id, &settings.local_mqtt_port)?;
    let auth = if settings.auth_mode == "Keyfile" {
        SshAuth::PrivateKey {
            path: non_empty(settings.ssh_key_file.trim()),
            private_key: None,
            passphrase: settings
                .ssh_password
                .expose_non_empty()
                .map(|value| SecretString::new(value.to_owned())),
        }
    } else {
        SshAuth::Password(SecretString::new(
            settings.ssh_password.expose_for_ui().to_owned(),
        ))
    };
    Ok(Some(SshTunnelOptions {
        host: settings.ssh_host.trim().to_owned(),
        port,
        username: settings.auth_username.trim().to_owned(),
        auth,
        host_key_policy: SshHostKeyPolicy::AcceptAnyInsecure,
        local_bind_port,
    }))
}

fn optional_port(
    connection_id: ConnectionId,
    value: &str,
) -> Result<Option<u16>, MqttCommandBuildError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    value
        .parse::<u16>()
        .map(Some)
        .map_err(|_| MqttCommandBuildError::InvalidPort { connection_id })
}

fn connected_connection_id(
    snapshot: &AppSnapshot,
    operation: MqttOperation,
) -> Result<ConnectionId, MqttCommandBuildError> {
    let connection_id = snapshot
        .selected_connection
        .ok_or(MqttCommandBuildError::NoSelectedConnection { operation })?;
    let connection = connection(snapshot, connection_id)
        .ok_or(MqttCommandBuildError::UnknownConnection { connection_id })?;
    if connection.state != ConnectionState::Connected {
        return Err(MqttCommandBuildError::NoOpenConnection {
            operation,
            connection_id,
        });
    }
    Ok(connection_id)
}

fn connection(snapshot: &AppSnapshot, connection_id: ConnectionId) -> Option<&ConnectionSummary> {
    snapshot
        .connections
        .iter()
        .find(|connection| connection.id == connection_id)
}

fn qos(qos: QosLevel) -> Qos {
    match qos {
        QosLevel::Zero => Qos::AtMostOnce,
        QosLevel::One => Qos::AtLeastOnce,
        QosLevel::Two => Qos::ExactlyOnce,
    }
}

fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}
