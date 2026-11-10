// Integration verification against the real OS keyring.
use correo_storage::current::{
    OsKeyringSecretStore, SecretKind, SecretMaterial, SecretReference, SecretStore,
};

// Distinct service so the probe never touches the app's real "secrets" entry.
fn probe_store() -> OsKeyringSecretStore {
    OsKeyringSecretStore::new("org.correomqtt.CorreoMQTT.session-probe")
}

fn reference() -> SecretReference {
    SecretReference {
        connection_id: "real-keychain-probe".to_owned(),
        kind: SecretKind::Password,
    }
}

struct CiLegacyCredential(keyring::Entry);

impl Drop for CiLegacyCredential {
    fn drop(&mut self) {
        let _ = self.0.delete_credential();
    }
}

fn seed_ci_legacy_credential() -> Option<CiLegacyCredential> {
    if std::env::var("CORREO_KEYRING_CI_SEED_LEGACY").as_deref() != Ok("1") {
        return None;
    }
    let entry = keyring::Entry::new("CorreoMQTT", "CorreoMQTT_MasterPassword")
        .expect("create legacy keyring entry");
    match entry.get_password() {
        Err(keyring::Error::NoEntry) => {}
        Ok(_) => panic!("refusing to overwrite an existing legacy keyring credential"),
        Err(error) => panic!("could not verify legacy keyring state: {error}"),
    }
    entry
        .set_password("session-test-master")
        .expect("seed legacy keyring credential");
    Some(CiLegacyCredential(entry))
}

#[test]
#[ignore = "touches the real OS keyring; run with --ignored"]
fn real_keyring_persists_across_store_instances() {
    let reference = reference();
    let _ = probe_store().delete(&reference); // clear any leftover

    // Write with one instance...
    probe_store()
        .put(&reference, &SecretMaterial::new("probe-secret-42"))
        .expect("put into real keyring");

    // ...read with a FRESH instance: proves real persistence, not caching.
    let restored = probe_store()
        .get(&reference)
        .expect("get from real keyring");
    assert_eq!(
        restored.map(SecretMaterial::expose_for_migration),
        Some("probe-secret-42".to_owned()),
        "secret must survive across separate store instances via the real keyring"
    );

    probe_store()
        .delete(&reference)
        .expect("delete from real keyring");
    assert_eq!(
        probe_store().get(&reference).expect("get after delete"),
        None,
        "secret must be gone after delete"
    );
}

#[test]
#[ignore = "reads the real OS keyring; CI seeds a disposable credential, or seed CorreoMQTT/CorreoMQTT_MasterPassword manually"]
fn migration_reads_java_master_password_from_real_keychain() {
    let _ci_credential = seed_ci_legacy_credential();
    let master = correo_storage::legacy::passwords::os_keyring_master_password();
    assert_eq!(
        master.as_deref(),
        Some("session-test-master"),
        "migration must read the Java master password from the real keychain"
    );
}
