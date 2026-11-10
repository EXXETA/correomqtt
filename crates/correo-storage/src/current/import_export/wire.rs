use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::{Result, StorageError};

use super::dto::{WireConnection, WireConnectionExport, WireConnectionExportOut};
use super::{
    ConnectionExport, ConnectionImport, ConnectionImportWarning, EncryptedConnectionExport, AES_GCM,
};
use crate::current::{
    Auth, ConnectionConfig, ImportedSecret, Lwt, MqttVersion, PasswordEncryption, Protocol, Proxy,
    Qos, SecretKind, SecretMaterial, SecretReference, TlsSsl,
};

pub(super) fn parse_connection_export_json(json: &str) -> Result<ConnectionExport> {
    let wire: WireConnectionExport = serde_json::from_str(json)
        .map_err(|source| StorageError::ConnectionExportJson { source })?;
    let mut warnings = Vec::new();
    record_extra_fields("connectionExport", &wire.extra, &mut warnings);

    match (wire.encryption_type, wire.encrypted_data) {
        (Some(encryption_type), Some(encrypted_data)) => {
            if wire.connection_config_dtos.is_some() {
                warnings.push(ConnectionImportWarning {
                    code: "connection_export_plaintext_ignored",
                    message:
                        "Encrypted connection export also contained plaintext connections; ignored"
                            .to_owned(),
                });
            }
            Ok(ConnectionExport::Encrypted(EncryptedConnectionExport {
                encryption: parse_encryption(&encryption_type)?,
                encrypted_data,
                warnings,
            }))
        }
        (Some(_), None) => Err(StorageError::InvalidConnectionExportPayload(
            "missing encryptedData",
        )),
        (None, Some(_)) => Err(StorageError::InvalidConnectionExportPayload(
            "missing encryptionType",
        )),
        (None, None) => {
            connection_import_from_wire(wire.connection_config_dtos.unwrap_or_default(), warnings)
                .map(ConnectionExport::Plain)
        }
    }
}

pub(super) fn connection_import_from_encrypted_json(
    json: &str,
    warnings: Vec<ConnectionImportWarning>,
) -> Result<ConnectionImport> {
    let wire_connections: Vec<WireConnection> = serde_json::from_str(json)
        .map_err(|source| StorageError::ConnectionExportJson { source })?;
    connection_import_from_wire(wire_connections, warnings)
}

pub(super) fn plain_export_json(import: &ConnectionImport) -> Result<String> {
    let wire = WireConnectionExportOut {
        encryption_type: None,
        encrypted_data: None,
        connection_config_dtos: Some(wire_connections_from_import(import, false)),
    };
    serialize_export(&wire)
}

pub(super) fn encrypted_payload_json(import: &ConnectionImport) -> Result<String> {
    serde_json::to_string(&wire_connections_from_import(import, true))
        .map_err(|source| StorageError::ConnectionExportJsonSerialize { source })
}

pub(super) fn encrypted_export_json(encrypted_data: String) -> Result<String> {
    let wire = WireConnectionExportOut {
        encryption_type: Some(AES_GCM),
        encrypted_data: Some(encrypted_data),
        connection_config_dtos: None,
    };
    serialize_export(&wire)
}

pub(super) fn encryption_name(encryption: PasswordEncryption) -> &'static str {
    match encryption {
        PasswordEncryption::AesGcmNoPadding => AES_GCM,
        PasswordEncryption::AesCbcPkcs5Padding => "AES/CBC/PKCS5Padding",
    }
}

fn connection_import_from_wire(
    connections: Vec<WireConnection>,
    mut warnings: Vec<ConnectionImportWarning>,
) -> Result<ConnectionImport> {
    let mut imported = ConnectionImport {
        connections: Vec::with_capacity(connections.len()),
        secrets: Vec::new(),
        warnings: Vec::new(),
    };

    for (index, mut connection) in connections.into_iter().enumerate() {
        let path = format!("connectionExport.connectionConfigDTOS[{index}]");
        record_extra_fields(&path, &connection.extra, &mut warnings);
        record_unmapped_value(
            &path,
            "connectionUISettings",
            &connection.connection_ui_settings,
            &mut warnings,
        );
        record_unmapped_value(
            &path,
            "publishListViewConfig",
            &connection.publish_list_view_config,
            &mut warnings,
        );
        record_unmapped_value(
            &path,
            "subscribeListViewConfig",
            &connection.subscribe_list_view_config,
            &mut warnings,
        );

        let id = require(connection.id.clone(), "connection export", "id")?;
        push_secret(
            &mut imported.secrets,
            &id,
            SecretKind::Password,
            connection.password.take(),
        );
        push_secret(
            &mut imported.secrets,
            &id,
            SecretKind::AuthPassword,
            connection.auth_password.take(),
        );
        push_secret(
            &mut imported.secrets,
            &id,
            SecretKind::SslKeystorePassword,
            connection.ssl_keystore_password.take(),
        );

        imported
            .connections
            .push(connection_from_wire(index, connection, &mut warnings)?);
    }

    imported.warnings = warnings;
    Ok(imported)
}

fn connection_from_wire(
    index: usize,
    connection: WireConnection,
    warnings: &mut Vec<ConnectionImportWarning>,
) -> Result<ConnectionConfig> {
    Ok(ConnectionConfig {
        id: require(connection.id, "connection export", "id")?,
        name: require(connection.name, "connection export", "name")?,
        // Only MQTT exists today; the .cqc wire format can carry protocol
        // when a second protocol lands (adding an optional field is
        // non-breaking).
        protocol: Protocol::Mqtt,
        url: require(connection.url, "connection export", "url")?,
        port: connection.port.unwrap_or(1883),
        client_id: connection.client_id,
        username: connection.username,
        clean_session: connection.clean_session,
        mqtt_version: mqtt_version(connection.mqtt_version.as_deref(), index, warnings),
        ssl: tls_mode(connection.ssl.as_deref(), index, warnings),
        ssl_keystore: connection.ssl_keystore,
        ssl_host_verification: connection.ssl_host_verification,
        proxy: proxy_mode(connection.proxy.as_deref(), index, warnings),
        ssh_host: connection.ssh_host,
        ssh_port: connection.ssh_port.unwrap_or(22),
        local_port: connection.local_port.filter(|port| *port != 0),
        auth: auth_mode(connection.auth.as_deref(), index, warnings),
        auth_username: connection.auth_username,
        auth_keyfile: connection.auth_keyfile,
        lwt: lwt_mode(connection.lwt.as_deref(), index, warnings),
        lwt_topic: connection.lwt_topic,
        lwt_qos: qos_value(connection.lwt_qos, index, warnings),
        lwt_retained: connection.lwt_retained,
        lwt_payload: connection.lwt_payload,
        connection_ui_settings: None,
        publish_list_view_config: None,
        subscribe_list_view_config: None,
        plugin_workflows: Vec::new(),
    })
}

fn wire_connections_from_import(
    import: &ConnectionImport,
    include_secrets: bool,
) -> Vec<WireConnection> {
    import
        .connections
        .iter()
        .map(|connection| WireConnection {
            id: Some(connection.id.clone()),
            name: Some(connection.name.clone()),
            url: Some(connection.url.clone()),
            port: Some(connection.port),
            client_id: connection.client_id.clone(),
            username: connection.username.clone(),
            password: include_secrets
                .then(|| secret_value(import, &connection.id, SecretKind::Password))
                .flatten(),
            clean_session: connection.clean_session,
            mqtt_version: Some(mqtt_version_name(connection.mqtt_version).to_owned()),
            ssl: Some(tls_name(connection.ssl).to_owned()),
            ssl_keystore: connection.ssl_keystore.clone(),
            ssl_keystore_password: include_secrets
                .then(|| secret_value(import, &connection.id, SecretKind::SslKeystorePassword))
                .flatten(),
            ssl_host_verification: connection.ssl_host_verification,
            proxy: Some(proxy_name(connection.proxy).to_owned()),
            ssh_host: connection.ssh_host.clone(),
            ssh_port: Some(connection.ssh_port),
            local_port: connection.local_port,
            auth: Some(auth_name(connection.auth).to_owned()),
            auth_username: connection.auth_username.clone(),
            auth_password: include_secrets
                .then(|| secret_value(import, &connection.id, SecretKind::AuthPassword))
                .flatten(),
            auth_keyfile: connection.auth_keyfile.clone(),
            lwt: Some(lwt_name(connection.lwt).to_owned()),
            lwt_topic: connection.lwt_topic.clone(),
            lwt_qos: connection.lwt_qos.map(qos_number),
            lwt_retained: connection.lwt_retained,
            lwt_payload: connection.lwt_payload.clone(),
            connection_ui_settings: None,
            publish_list_view_config: None,
            subscribe_list_view_config: None,
            extra: BTreeMap::new(),
        })
        .collect()
}

fn secret_value(
    import: &ConnectionImport,
    connection_id: &str,
    kind: SecretKind,
) -> Option<String> {
    import
        .secrets
        .iter()
        .find(|secret| {
            secret.reference.connection_id == connection_id && secret.reference.kind == kind
        })
        .map(|secret| secret.value.clone().expose_for_migration())
}

fn push_secret(
    secrets: &mut Vec<ImportedSecret>,
    connection_id: &str,
    kind: SecretKind,
    value: Option<String>,
) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        secrets.push(ImportedSecret {
            reference: SecretReference {
                connection_id: connection_id.to_owned(),
                kind,
            },
            value: SecretMaterial::new(value),
        });
    }
}

fn serialize_export<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string_pretty(value)
        .map_err(|source| StorageError::ConnectionExportJsonSerialize { source })
}

fn parse_encryption(value: &str) -> Result<PasswordEncryption> {
    match value {
        AES_GCM => Ok(PasswordEncryption::AesGcmNoPadding),
        other => Err(StorageError::UnsupportedConnectionExportEncryption(
            other.to_owned(),
        )),
    }
}

fn mqtt_version(
    value: Option<&str>,
    index: usize,
    warnings: &mut Vec<ConnectionImportWarning>,
) -> MqttVersion {
    match value {
        Some("MQTT_3_1_1" | "MQTT311" | "MQTT 3.1.1") => MqttVersion::Mqtt311,
        Some("MQTT_5_0" | "MQTT5" | "MQTT 5") | None => MqttVersion::Mqtt50,
        Some(other) => {
            unknown_enum_warning("mqttVersion", other, index, "MQTT_5_0", warnings);
            MqttVersion::Mqtt50
        }
    }
}

fn tls_mode(
    value: Option<&str>,
    index: usize,
    warnings: &mut Vec<ConnectionImportWarning>,
) -> TlsSsl {
    match normalized(value).as_deref() {
        Some("OFF") | None => TlsSsl::Off,
        Some("ON" | "KEYSTORE" | "SSL" | "TLS") => TlsSsl::Keystore,
        Some(other) => {
            unknown_enum_warning("ssl", other, index, "OFF", warnings);
            TlsSsl::Off
        }
    }
}

fn proxy_mode(
    value: Option<&str>,
    index: usize,
    warnings: &mut Vec<ConnectionImportWarning>,
) -> Proxy {
    match normalized(value).as_deref() {
        Some("OFF") | None => Proxy::Off,
        Some("SSH") => Proxy::Ssh,
        Some(other) => {
            unknown_enum_warning("proxy", other, index, "OFF", warnings);
            Proxy::Off
        }
    }
}

fn auth_mode(
    value: Option<&str>,
    index: usize,
    warnings: &mut Vec<ConnectionImportWarning>,
) -> Auth {
    match normalized(value).as_deref() {
        Some("OFF") | None => Auth::Off,
        Some("PASSWORD") => Auth::Password,
        Some("KEY" | "KEYFILE") => Auth::Keyfile,
        Some(other) => {
            unknown_enum_warning("auth", other, index, "OFF", warnings);
            Auth::Off
        }
    }
}

fn lwt_mode(value: Option<&str>, index: usize, warnings: &mut Vec<ConnectionImportWarning>) -> Lwt {
    match normalized(value).as_deref() {
        Some("OFF") | None => Lwt::Off,
        Some("ON") => Lwt::On,
        Some(other) => {
            unknown_enum_warning("lwt", other, index, "OFF", warnings);
            Lwt::Off
        }
    }
}

fn qos_value(
    value: Option<u8>,
    index: usize,
    warnings: &mut Vec<ConnectionImportWarning>,
) -> Option<Qos> {
    match value {
        Some(value) => Qos::from_legacy(value).or_else(|| {
            warnings.push(ConnectionImportWarning {
                code: "connection_export_qos_unknown",
                message: format!(
                    "Connection export connectionConfigDTOS[{index}].lwtQoS={value} could not be mapped"
                ),
            });
            None
        }),
        None => None,
    }
}

fn unknown_enum_warning(
    field: &str,
    value: &str,
    index: usize,
    default: &str,
    warnings: &mut Vec<ConnectionImportWarning>,
) {
    warnings.push(ConnectionImportWarning {
        code: "connection_export_enum_unknown",
        message: format!(
            "Connection export connectionConfigDTOS[{index}].{field}={value} defaulted to {default}"
        ),
    });
}

fn record_extra_fields(
    owner_path: &str,
    extra: &BTreeMap<String, Value>,
    warnings: &mut Vec<ConnectionImportWarning>,
) {
    warnings.extend(extra.keys().map(|field| ConnectionImportWarning {
        code: "unsupported_connection_export_field",
        message: format!("Unsupported connection export field ignored: {owner_path}.{field}"),
    }));
}

fn record_unmapped_value(
    owner_path: &str,
    field: &str,
    value: &Option<Value>,
    warnings: &mut Vec<ConnectionImportWarning>,
) {
    if value.is_some() {
        warnings.push(ConnectionImportWarning {
            code: "unsupported_connection_export_field",
            message: format!("Unsupported connection export field ignored: {owner_path}.{field}"),
        });
    }
}

fn normalized(value: Option<&str>) -> Option<String> {
    value.map(|value| value.trim().to_ascii_uppercase())
}

fn require(value: Option<String>, record: &'static str, field: &'static str) -> Result<String> {
    value.ok_or(StorageError::MissingField { record, field })
}

fn mqtt_version_name(value: MqttVersion) -> &'static str {
    match value {
        MqttVersion::Mqtt311 => "MQTT_3_1_1",
        MqttVersion::Mqtt50 => "MQTT_5_0",
    }
}

fn tls_name(value: TlsSsl) -> &'static str {
    match value {
        TlsSsl::Off => "OFF",
        TlsSsl::Keystore => "KEYSTORE",
    }
}

fn proxy_name(value: Proxy) -> &'static str {
    match value {
        Proxy::Off => "OFF",
        Proxy::Ssh => "SSH",
    }
}

fn auth_name(value: Auth) -> &'static str {
    match value {
        Auth::Off => "OFF",
        Auth::Password => "PASSWORD",
        Auth::Keyfile => "KEYFILE",
    }
}

fn lwt_name(value: Lwt) -> &'static str {
    match value {
        Lwt::Off => "OFF",
        Lwt::On => "ON",
    }
}

fn qos_number(value: Qos) -> u8 {
    match value {
        Qos::AtMostOnce => 0,
        Qos::AtLeastOnce => 1,
        Qos::ExactlyOnce => 2,
    }
}
