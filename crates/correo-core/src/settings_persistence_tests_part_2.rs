#[test]
fn failed_broker_config_save_restores_previous_secret() {
    use correo_storage::current::{SecretKind, SecretMaterial, SecretReference, SecretStore};

    let secret_store = MemorySecretStore::default();
    let reference = SecretReference {
        connection_id: "built-in-broker".to_owned(),
        kind: SecretKind::Password,
    };
    secret_store
        .put(&reference, &SecretMaterial::new("old-broker-secret"))
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let blocked_root = temp.path().join("blocked-root");
    std::fs::write(&blocked_root, "not a directory").unwrap();
    let store = ConfigStore::new(blocked_root);

    let result = super::save_built_in_broker(
        &store,
        &secret_store,
        crate::BuiltInBrokerPersistenceSnapshot {
            port: "1883".to_owned(),
            credentials_enabled: true,
            username: "broker".to_owned(),
            password: "new-broker-secret".to_owned(),
        },
    );

    assert!(result.is_err());
    assert_eq!(
        secret_store
            .get(&reference)
            .unwrap()
            .map(SecretMaterial::expose_for_migration),
        Some("old-broker-secret".to_owned())
    );
}

#[derive(Default)]
struct MemorySecretStore(
    std::cell::RefCell<std::collections::BTreeMap<String, correo_storage::current::SecretMaterial>>,
);

impl correo_storage::current::SecretStore for MemorySecretStore {
    fn put(
        &self,
        reference: &correo_storage::current::SecretReference,
        value: &correo_storage::current::SecretMaterial,
    ) -> correo_storage::Result<()> {
        self.0
            .borrow_mut()
            .insert(reference.keyring_account(), value.clone());
        Ok(())
    }

    fn get(
        &self,
        reference: &correo_storage::current::SecretReference,
    ) -> correo_storage::Result<Option<correo_storage::current::SecretMaterial>> {
        Ok(self.0.borrow().get(&reference.keyring_account()).cloned())
    }

    fn delete(
        &self,
        reference: &correo_storage::current::SecretReference,
    ) -> correo_storage::Result<()> {
        self.0.borrow_mut().remove(&reference.keyring_account());
        Ok(())
    }
}

struct FailingSecretStore;

impl correo_storage::current::SecretStore for FailingSecretStore {
    fn put(
        &self,
        _reference: &correo_storage::current::SecretReference,
        _value: &correo_storage::current::SecretMaterial,
    ) -> correo_storage::Result<()> {
        Err(correo_storage::StorageError::SecretStore {
            operation: "write",
            reference: "injected".to_owned(),
            message: "secret store unavailable".to_owned(),
        })
    }

    fn get(
        &self,
        _reference: &correo_storage::current::SecretReference,
    ) -> correo_storage::Result<Option<correo_storage::current::SecretMaterial>> {
        Ok(None)
    }

    fn delete(
        &self,
        _reference: &correo_storage::current::SecretReference,
    ) -> correo_storage::Result<()> {
        Ok(())
    }
}

#[test]
fn failed_secret_write_does_not_persist_connection() {
    let temp = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(temp.path());
    let mut settings = connection_settings("Broker", "localhost");
    settings.password = crate::SecretInput::new("hunter2");

    let result = super::save_connection_settings(
        &store,
        &FailingSecretStore,
        "connection-01".to_owned(),
        settings,
    );

    assert!(result.is_err(), "secret failure must surface as an error");
    // Secrets are written before the config, so a failed secret write leaves
    // no config file (or an empty one) — never a connection without credentials.
    let persisted = store
        .load()
        .map(|config| config.connections.len())
        .unwrap_or(0);
    assert_eq!(
        persisted, 0,
        "connection must not be persisted when its secret write failed"
    );
}

#[test]
fn failed_config_write_rolls_back_secret_changes() {
    use correo_storage::current::{
        ImportedSecret, OsKeyringSecretStore, SecretKind, SecretMaterial, SecretReference,
        SecretStore,
    };

    keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    let secret_store = OsKeyringSecretStore::new("test.rollback.config");
    let reference = SecretReference {
        connection_id: "c1".to_owned(),
        kind: SecretKind::Password,
    };
    secret_store
        .put(&reference, &SecretMaterial::new("old-secret"))
        .unwrap();

    let new_put = ImportedSecret {
        reference: reference.clone(),
        value: SecretMaterial::new("new-secret"),
    };
    let result =
        super::apply_secrets_then_save(&secret_store, std::slice::from_ref(&new_put), &[], || {
            Err(correo_storage::StorageError::SecretStore {
                operation: "write",
                reference: "config".to_owned(),
                message: "injected config failure".to_owned(),
            })
        });

    assert!(result.is_err(), "config failure must surface as an error");
    let restored = secret_store.get(&reference).unwrap();
    assert_eq!(
        restored.map(SecretMaterial::expose_for_migration),
        Some("old-secret".to_owned()),
        "secret must roll back to its previous value when the config write fails"
    );
}
