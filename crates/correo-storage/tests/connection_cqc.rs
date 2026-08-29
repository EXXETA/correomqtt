use std::path::{Path, PathBuf};

use correo_storage::current::ImportedSecret;
use correo_storage::current::{
    connection_export_encrypted_json, connection_export_plain_json, decrypt_connection_export,
    import_connection_export_json, read_connection_export, write_plain_connection_export, Auth,
    ConnectionConfig, ConnectionExport, ConnectionImport, Lwt, MqttVersion, Protocol, Proxy, Qos,
    SecretKind, SecretMaterial, SecretReference, TlsSsl,
};
use correo_storage::StorageError;
use serde_json::Value;

const EXPORT_PASSWORD: &str = "synthetic-export-password";

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

#[test]
fn imports_java_plain_connection_export_fixture() {
    let export = read_connection_export(fixture("legacy_profile/exports/connections.cqc")).unwrap();
    let ConnectionExport::Plain(import) = export else {
        panic!("expected plain connection export");
    };

    assert_eq!(import.connections.len(), 1);
    assert!(import.secrets.is_empty());
    assert!(import.warnings.is_empty());

    let connection = &import.connections[0];
    assert_eq!(connection.id, "exported-broker-01");
    assert_eq!(connection.name, "Synthetic Exported Broker");
    assert_eq!(connection.url, "export.example.invalid");
    assert_eq!(connection.client_id.as_deref(), Some("correo-export"));
    assert_eq!(
        connection.username.as_deref(),
        Some("synthetic-export-user")
    );
    assert_eq!(connection.mqtt_version, MqttVersion::Mqtt50);
    assert_eq!(connection.ssl, TlsSsl::Off);
    assert_eq!(connection.proxy, Proxy::Off);
    assert_eq!(connection.auth, Auth::Off);
    assert_eq!(connection.lwt, Lwt::Off);
}

#[test]
fn cqc_import_records_unsupported_fields_as_warnings() {
    let import = import_connection_export_json(
        r#"{
          "futureRoot": true,
          "connectionConfigDTOS": [
            {
              "id": "warning-broker",
              "name": "Warning Broker",
              "url": "localhost",
              "mqttVersion": "MQTT_7_0",
              "lwtQoS": 9,
              "connectionUISettings": { "selectedTab": "publish" },
              "futureConnectionField": "not yet mapped"
            }
          ]
        }"#,
        None,
    )
    .unwrap();

    assert_eq!(import.connections.len(), 1);
    assert_eq!(import.connections[0].mqtt_version, MqttVersion::Mqtt50);
    assert_eq!(import.connections[0].lwt_qos, None);

    let messages = import
        .warnings
        .iter()
        .map(|warning| warning.message.as_str())
        .collect::<Vec<_>>();
    assert!(messages
        .iter()
        .any(|message| message.contains("connectionExport.futureRoot")));
    assert!(messages
        .iter()
        .any(|message| message.contains("futureConnectionField")));
    assert!(messages
        .iter()
        .any(|message| message.contains("connectionUISettings")));
    assert!(import
        .warnings
        .iter()
        .any(|warning| warning.code == "connection_export_enum_unknown"));
    assert!(import
        .warnings
        .iter()
        .any(|warning| warning.code == "connection_export_qos_unknown"));
}

#[test]
fn plain_export_writes_connection_config_dtos_without_secrets() {
    let import = sample_import();
    let json = connection_export_plain_json(&import).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let connection = &value["connectionConfigDTOS"][0];

    assert!(value.get("encryptionType").is_none());
    assert!(value.get("encryptedData").is_none());
    assert_eq!(connection["id"], "roundtrip-broker");
    assert_eq!(connection["mqttVersion"], "MQTT_5_0");
    assert_eq!(connection["auth"], "PASSWORD");
    assert_eq!(connection["lwtQoS"], 1);
    assert!(connection.get("password").is_none());
    assert!(connection.get("authPassword").is_none());
    assert!(connection.get("sslKeystorePassword").is_none());
    assert!(!json.contains("synthetic-mqtt-secret"));

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("connections.cqc");
    write_plain_connection_export(&path, &import).unwrap();
    let ConnectionExport::Plain(read_back) = read_connection_export(&path).unwrap() else {
        panic!("expected plain connection export");
    };
    assert_eq!(read_back.connections, import.connections);
    assert!(read_back.secrets.is_empty());
}

#[test]
fn encrypted_export_import_round_trips_aes_gcm_with_synthetic_passwords() {
    let import = sample_import();
    let json = connection_export_encrypted_json(&import, EXPORT_PASSWORD).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["encryptionType"], "AES/GCM/NoPadding");
    assert!(value["encryptedData"].as_str().unwrap().len() > 64);
    assert!(value.get("connectionConfigDTOS").is_none());
    assert!(!json.contains("synthetic-mqtt-secret"));
    assert!(!json.contains("synthetic-auth-secret"));
    assert!(!json.contains("synthetic-keystore-secret"));

    let ConnectionExport::Encrypted(export) =
        correo_storage::current::parse_connection_export_json(&json).unwrap()
    else {
        panic!("expected encrypted connection export");
    };
    let decrypted = decrypt_connection_export(&export, EXPORT_PASSWORD).unwrap();
    assert_eq!(decrypted.connections, import.connections);
    assert_eq!(
        secret(&decrypted, SecretKind::Password).as_deref(),
        Some("synthetic-mqtt-secret")
    );
    assert_eq!(
        secret(&decrypted, SecretKind::AuthPassword).as_deref(),
        Some("synthetic-auth-secret")
    );
    assert_eq!(
        secret(&decrypted, SecretKind::SslKeystorePassword).as_deref(),
        Some("synthetic-keystore-secret")
    );

    let direct_import = import_connection_export_json(&json, Some(EXPORT_PASSWORD)).unwrap();
    assert_eq!(direct_import.connections, import.connections);
    assert!(matches!(
        decrypt_connection_export(&export, "wrong-synthetic-password"),
        Err(StorageError::ConnectionExportDecryption)
    ));
}

fn sample_import() -> ConnectionImport {
    ConnectionImport {
        connections: vec![ConnectionConfig {
            id: "roundtrip-broker".to_owned(),
            name: "Round-trip Broker".to_owned(),
            protocol: Protocol::Mqtt,
            url: "mqtt.example.invalid".to_owned(),
            port: 8883,
            client_id: Some("correo-roundtrip".to_owned()),
            username: Some("synthetic-user".to_owned()),
            clean_session: true,
            mqtt_version: MqttVersion::Mqtt50,
            ssl: TlsSsl::Keystore,
            ssl_keystore: Some("/synthetic/path/keystore.p12".to_owned()),
            ssl_host_verification: true,
            proxy: Proxy::Ssh,
            ssh_host: Some("ssh.example.invalid".to_owned()),
            ssh_port: 22,
            local_port: Some(11883),
            auth: Auth::Password,
            auth_username: Some("synthetic-ssh-user".to_owned()),
            auth_keyfile: None,
            lwt: Lwt::On,
            lwt_topic: Some("status/roundtrip".to_owned()),
            lwt_qos: Some(Qos::AtLeastOnce),
            lwt_retained: true,
            lwt_payload: Some("offline".to_owned()),
            connection_ui_settings: None,
            publish_list_view_config: None,
            subscribe_list_view_config: None,
            plugin_workflows: Vec::new(),
        }],
        secrets: vec![
            imported_secret(SecretKind::Password, "synthetic-mqtt-secret"),
            imported_secret(SecretKind::AuthPassword, "synthetic-auth-secret"),
            imported_secret(SecretKind::SslKeystorePassword, "synthetic-keystore-secret"),
        ],
        warnings: Vec::new(),
    }
}

fn imported_secret(kind: SecretKind, value: &str) -> ImportedSecret {
    ImportedSecret {
        reference: SecretReference {
            connection_id: "roundtrip-broker".to_owned(),
            kind,
        },
        value: SecretMaterial::new(value),
    }
}

fn secret(import: &ConnectionImport, kind: SecretKind) -> Option<String> {
    import
        .secrets
        .iter()
        .find(|secret| {
            secret.reference.connection_id == "roundtrip-broker" && secret.reference.kind == kind
        })
        .map(|secret| secret.value.clone().expose_for_migration())
}
