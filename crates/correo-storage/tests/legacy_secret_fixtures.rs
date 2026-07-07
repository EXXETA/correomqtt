use correo_storage::current::{
    redact_script_log_text, ScriptExecution, ScriptExecutionStatus, ScriptLogLevel,
    ScriptLogRecord, ScriptStore,
};
use correo_storage::legacy::passwords::{LegacyPasswords, SecretKind};
use correo_storage::migration::connection_secrets;
use correo_storage::StorageError;
use std::path::{Path, PathBuf};

const MASTER_PASSWORD: &str = "synthetic-master-passphrase";

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

#[test]
fn decrypts_current_aes_gcm_password_fixture() {
    let passwords = LegacyPasswords::read_from(fixture("legacy_profile/passwords.json"))
        .unwrap()
        .decrypt(MASTER_PASSWORD)
        .unwrap();

    let secrets = connection_secrets(&passwords, "local-broker-01");
    assert_eq!(secrets.len(), 3);
    assert!(secrets.contains(&(SecretKind::Password, "synthetic-mqtt-password")));
    assert!(secrets.contains(&(SecretKind::AuthPassword, "synthetic-ssh-password")));
    assert!(secrets.contains(&(
        SecretKind::SslKeystorePassword,
        "synthetic-keystore-password"
    )));
}

#[test]
fn decrypts_legacy_aes_cbc_password_fixture() {
    let passwords = LegacyPasswords::read_from(fixture("password_formats/passwords_cbc.json"))
        .unwrap()
        .decrypt(MASTER_PASSWORD)
        .unwrap();

    assert_eq!(
        passwords
            .get("local-broker-01_password")
            .map(String::as_str),
        Some("synthetic-mqtt-password")
    );
    assert_eq!(
        passwords
            .get("local-broker-01_auth_password")
            .map(String::as_str),
        Some("synthetic-ssh-password")
    );
    assert_eq!(
        passwords
            .get("local-broker-01_ssl_keystore_password")
            .map(String::as_str),
        Some("synthetic-keystore-password")
    );
}

#[test]
fn script_store_crud_tracks_dirty_state_and_redacts_logs() {
    let temp = tempfile::tempdir().unwrap();
    let store = ScriptStore::new(temp.path());

    let script = store
        .create_script("alerts/publish.js", "logger.info('ok');")
        .unwrap();
    assert_eq!(script.name, "publish.js");
    assert_eq!(store.list_scripts().unwrap().len(), 1);
    assert!(
        !store
            .dirty_state("alerts/publish.js", "logger.info('ok');")
            .unwrap()
            .dirty
    );
    assert!(
        store
            .dirty_state("alerts/publish.js", "logger.info('changed');")
            .unwrap()
            .dirty
    );

    store
        .update_script("alerts/publish.js", "logger.info('changed');")
        .unwrap();
    let renamed = store
        .rename_script("alerts/publish.js", "alerts/publish_renamed.js")
        .unwrap();
    assert_eq!(
        renamed.relative_path,
        Path::new("alerts/publish_renamed.js")
    );
    assert!(matches!(
        store.create_script("../escape.js", ""),
        Err(StorageError::InvalidScriptFileName(_))
    ));

    let execution = ScriptExecution {
        execution_id: "execution-002".to_owned(),
        script_name: "publish_renamed.js".to_owned(),
        script_path: Path::new("alerts/publish_renamed.js").to_path_buf(),
        connection_id: Some("local-broker-01".to_owned()),
        status: ScriptExecutionStatus::Running,
        error: None,
        started_at: Some("2026-06-08T17:20:00.000".to_owned()),
        ended_at: None,
        duration_ms: None,
        cancelled: false,
        log_path: None,
    };
    store
        .save_execution("alerts/publish_renamed.js", &execution)
        .unwrap();
    assert_eq!(
        store
            .load_executions("alerts/publish_renamed.js")
            .unwrap()
            .first()
            .unwrap()
            .execution_id,
        "execution-002"
    );

    for (sequence, message) in [
        "INFO first line",
        "password=synthetic-runtime-password",
        "private key material: synthetic-key-material",
    ]
    .into_iter()
    .enumerate()
    {
        store
            .append_log_record(
                "alerts/publish_renamed.js",
                &ScriptLogRecord {
                    execution_id: "execution-002".to_owned(),
                    sequence: sequence as u64,
                    timestamp: None,
                    level: ScriptLogLevel::Info,
                    message: message.to_owned(),
                },
            )
            .unwrap();
    }
    let log = store
        .load_log("alerts/publish_renamed.js", "execution-002", 2)
        .unwrap();
    assert_eq!(log.records.len(), 2);
    assert_eq!(log.truncated_count, 1);
    assert!(log.records.iter().all(|record| !record
        .message
        .contains("synthetic-runtime-password")
        && !record.message.contains("synthetic-key-material")));

    store.delete_script("alerts/publish_renamed.js").unwrap();
    assert!(matches!(
        store.load_script("alerts/publish_renamed.js"),
        Err(StorageError::ScriptNotFound(_))
    ));
}

#[test]
fn redacts_sensitive_script_log_shapes() {
    let redacted = redact_script_log_text(
        "password=synthetic-password\nexport password: synthetic-export\n-----BEGIN PRIVATE KEY-----",
    );

    assert!(redacted.contains("password= [REDACTED]"));
    assert!(redacted.contains("export password: [REDACTED]"));
    assert!(redacted.contains("[REDACTED KEY MATERIAL]"));
    assert!(!redacted.contains("synthetic-password"));
    assert!(!redacted.contains("synthetic-export"));
}
