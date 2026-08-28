use super::*;

fn mock_store(service: &str) -> OsKeyringSecretStore {
    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    OsKeyringSecretStore::new(service)
}

fn reference(connection_id: &str, kind: SecretKind) -> SecretReference {
    SecretReference {
        connection_id: connection_id.to_owned(),
        kind,
    }
}

fn imported(connection_id: &str, kind: SecretKind, value: &str) -> ImportedSecret {
    ImportedSecret {
        reference: reference(connection_id, kind),
        value: SecretMaterial::new(value),
    }
}

// Exercises the macOS blob layout directly (platform-independent via the
// MapBacked methods) so the single-entry invariant is tested on any host.
#[test]
fn blob_map_roundtrip_uses_single_entry() {
    let store = mock_store("test.blob.roundtrip");
    let password = reference("c1", SecretKind::Password);
    let auth = reference("c1", SecretKind::AuthPassword);

    store
        .map_apply(
            &[
                imported("c1", SecretKind::Password, "secret-a"),
                imported("c1", SecretKind::AuthPassword, "secret-b"),
            ],
            &[],
        )
        .expect("apply");

    let raw = store
        .blob_entry()
        .expect("entry")
        .get_password()
        .expect("single blob entry holds all secrets");
    let map: BTreeMap<String, String> = serde_json::from_str(&raw).expect("json blob");
    assert_eq!(map.len(), 2);

    assert_eq!(
        store
            .map_get_all(&[password.clone(), auth.clone()])
            .expect("get_all")
            .into_iter()
            .map(|value| value.map(SecretMaterial::expose_for_migration))
            .collect::<Vec<_>>(),
        vec![Some("secret-a".to_owned()), Some("secret-b".to_owned())]
    );

    store.map_delete(&password).expect("delete");
    assert_eq!(store.map_get(&password).expect("get after delete"), None);
    assert_eq!(
        store.map_get(&auth).expect("sibling survives delete"),
        Some(SecretMaterial::new("secret-b"))
    );

    store
        .map_apply(&[], std::slice::from_ref(&auth))
        .expect("delete last via apply");
    assert_eq!(store.map_get(&auth).expect("empty store reads none"), None);
}

// Exercises the Windows/Linux per-secret layout directly. One credential
// per secret keeps Windows under CRED_MAX_CREDENTIAL_BLOB_SIZE.
#[test]
fn per_entry_roundtrip_stores_one_credential_per_secret() {
    let store = mock_store("test.entries.roundtrip");
    let password = reference("c1", SecretKind::Password);

    store
        .entry_put(&password, &SecretMaterial::new("secret-a"))
        .expect("put");
    assert_eq!(
        store.entry_get(&password).expect("get"),
        Some(SecretMaterial::new("secret-a"))
    );

    store.entry_delete(&password).expect("delete");
    assert_eq!(store.entry_get(&password).expect("get after delete"), None);
    store.entry_delete(&password).expect("delete is idempotent");
}

#[test]
fn encrypted_file_store_roundtrip_batching_and_wrong_password() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("secrets.enc");
    let store = EncryptedFileSecretStore::new(&path, "correct horse");
    let password = reference("c1", SecretKind::Password);
    let auth = reference("c1", SecretKind::AuthPassword);

    store
        .apply(
            &[imported("c1", SecretKind::Password, "secret-a")],
            std::slice::from_ref(&auth),
        )
        .expect("apply with put and no-op delete");
    assert!(path.exists());
    assert_eq!(
        store.get(&password).expect("get"),
        Some(SecretMaterial::new("secret-a"))
    );

    let wrong = EncryptedFileSecretStore::new(&path, "wrong password");
    assert!(matches!(
        wrong.get(&password),
        Err(StorageError::PasswordDecryption)
    ));

    store.apply(&[], &[password]).expect("delete last");
    assert!(!path.exists());
}

#[test]
fn encrypted_file_store_reads_legacy_kdf_and_writes_versioned_parameters() {
    use aes_gcm::{aead::Aead, KeyInit};
    use base64::Engine;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("secrets.enc");
    let salt = [7u8; 16];
    let nonce = [9u8; 12];
    let mut key = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(b"correct horse", &salt, 100_000, &mut key);
    let plaintext = serde_json::to_vec(&BTreeMap::from([(
        "connection:c1:password".to_owned(),
        "legacy-secret".to_owned(),
    )]))
    .unwrap();
    let ciphertext = aes_gcm::Aes256Gcm::new_from_slice(&key)
        .unwrap()
        .encrypt(aes_gcm::Nonce::from_slice(&nonce), plaintext.as_slice())
        .unwrap();
    let engine = base64::engine::general_purpose::STANDARD;
    std::fs::write(
        &path,
        serde_json::json!({
            "salt": engine.encode(salt),
            "nonce": engine.encode(nonce),
            "ciphertext": engine.encode(ciphertext),
        })
        .to_string(),
    )
    .unwrap();

    let store = EncryptedFileSecretStore::new(&path, "correct horse");
    let password = reference("c1", SecretKind::Password);
    assert_eq!(
        store.get(&password).unwrap(),
        Some(SecretMaterial::new("legacy-secret"))
    );

    store
        .put(&password, &SecretMaterial::new("current-secret"))
        .unwrap();
    let reopened = EncryptedFileSecretStore::new(&path, "correct horse");
    assert_eq!(
        reopened.get(&password).unwrap(),
        Some(SecretMaterial::new("current-secret"))
    );
    let file: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(file["version"], 2);
    assert_eq!(file["kdf"]["algorithm"], "pbkdf2-hmac-sha256");
    assert_eq!(file["kdf"]["iterations"], 600_000);
}

#[cfg(not(debug_assertions))]
#[test]
#[ignore = "release performance check; run explicitly with --release --ignored"]
fn encrypted_file_store_release_roundtrip_stays_within_runtime_budget() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("secrets.enc");
    let store = EncryptedFileSecretStore::new(&path, "correct horse");
    let password = reference("c1", SecretKind::Password);
    let started = std::time::Instant::now();

    store
        .put(&password, &SecretMaterial::new("current-secret"))
        .unwrap();
    assert_eq!(
        store.get(&password).unwrap(),
        Some(SecretMaterial::new("current-secret"))
    );
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "representative release write/read must remain practical"
    );
}

#[test]
fn encrypted_file_store_rejects_corrupt_nonce_without_panicking() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("secrets.enc");
    // Valid base64 but only 8 bytes where a 12-byte AES-GCM nonce is required.
    std::fs::write(
        &path,
        r#"{"salt":"AAAAAAAAAAAAAAAAAAAAAA==","nonce":"AAAAAAAAAAA=","ciphertext":"AAAA"}"#,
    )
    .expect("write corrupt file");

    let store = EncryptedFileSecretStore::new(&path, "correct horse");
    let result = store.get(&reference("c1", SecretKind::Password));
    assert!(
        matches!(result, Err(StorageError::SecretStore { .. })),
        "corrupt nonce must be a recoverable error, got {result:?}"
    );
}
