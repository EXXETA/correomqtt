use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use std::fmt;

use super::config_root;
use crate::{Result, StorageError};

pub const KEYRING_SERVICE: &str = "org.correomqtt.CorreoMQTT";
pub const KEYRING_BLOB_ACCOUNT: &str = "secrets";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordFile {
    pub encryption: PasswordEncryption,
    pub encrypted_payload: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordEncryption {
    AesGcmNoPadding,
    AesCbcPkcs5Padding,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretMaterial(String);

impl SecretMaterial {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    pub fn expose_for_migration(self) -> String {
        self.0
    }
}

impl fmt::Debug for SecretMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretMaterial(<redacted>)")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedSecret {
    pub reference: SecretReference,
    pub value: SecretMaterial,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretReference {
    pub connection_id: String,
    pub kind: SecretKind,
}

impl SecretReference {
    pub fn keyring_account(&self) -> String {
        format!("connection:{}:{}", self.connection_id, self.kind.label())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretKind {
    Password,
    AuthPassword,
    SslKeystorePassword,
}

impl SecretKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::AuthPassword => "auth_password",
            Self::SslKeystorePassword => "ssl_keystore_password",
        }
    }
}

pub trait SecretStore {
    fn put(&self, reference: &SecretReference, value: &SecretMaterial) -> Result<()>;
    fn get(&self, reference: &SecretReference) -> Result<Option<SecretMaterial>>;
    fn delete(&self, reference: &SecretReference) -> Result<()>;

    fn get_all(&self, references: &[SecretReference]) -> Result<Vec<Option<SecretMaterial>>> {
        references
            .iter()
            .map(|reference| self.get(reference))
            .collect()
    }

    /// Applies puts and deletes in one operation; map-backed stores batch
    /// this into a single load + save.
    fn apply(&self, puts: &[ImportedSecret], deletes: &[SecretReference]) -> Result<()> {
        for secret in puts {
            self.put(&secret.reference, &secret.value)?;
        }
        for reference in deletes {
            self.delete(reference)?;
        }
        Ok(())
    }

    fn put_all(&self, secrets: &[ImportedSecret]) -> Result<usize> {
        self.apply(secrets, &[])?;
        Ok(secrets.len())
    }
}

/// Backends that persist the full account→secret map in one place. CRUD and
/// batching are provided once here so keyring-blob and encrypted-file
/// backends cannot drift apart.
trait MapBacked {
    fn load_map(&self) -> Result<BTreeMap<String, String>>;
    fn save_map(&self, map: &BTreeMap<String, String>) -> Result<()>;

    fn map_put(&self, reference: &SecretReference, value: &SecretMaterial) -> Result<()> {
        let mut map = self.load_map()?;
        map.insert(
            reference.keyring_account(),
            value.expose_secret().to_owned(),
        );
        self.save_map(&map)
    }

    fn map_get(&self, reference: &SecretReference) -> Result<Option<SecretMaterial>> {
        Ok(self
            .load_map()?
            .get(&reference.keyring_account())
            .map(SecretMaterial::new))
    }

    fn map_delete(&self, reference: &SecretReference) -> Result<()> {
        let mut map = self.load_map()?;
        if map.remove(&reference.keyring_account()).is_some() {
            self.save_map(&map)?;
        }
        Ok(())
    }

    fn map_get_all(&self, references: &[SecretReference]) -> Result<Vec<Option<SecretMaterial>>> {
        let map = self.load_map()?;
        Ok(references
            .iter()
            .map(|reference| {
                map.get(&reference.keyring_account())
                    .map(SecretMaterial::new)
            })
            .collect())
    }

    fn map_apply(&self, puts: &[ImportedSecret], deletes: &[SecretReference]) -> Result<()> {
        if puts.is_empty() && deletes.is_empty() {
            return Ok(());
        }
        let mut map = self.load_map()?;
        for secret in puts {
            map.insert(
                secret.reference.keyring_account(),
                secret.value.expose_secret().to_owned(),
            );
        }
        let mut removed = false;
        for reference in deletes {
            removed |= map.remove(&reference.keyring_account()).is_some();
        }
        if !puts.is_empty() || removed {
            self.save_map(&map)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct EntryCache {
    entries: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, std::sync::Arc<keyring::Entry>>>,
    >,
}

#[derive(Clone, Debug)]
pub struct OsKeyringSecretStore {
    service: String,
    cache: EntryCache,
}

impl OsKeyringSecretStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            cache: EntryCache::default(),
        }
    }

    fn entry(&self, account: &str) -> Result<std::sync::Arc<keyring::Entry>> {
        let mut entries = self.cache.entries.lock().expect("entry cache poisoned");
        if let Some(entry) = entries.get(account) {
            return Ok(entry.clone());
        }
        let entry = std::sync::Arc::new(
            keyring::Entry::new(&self.service, account)
                .map_err(|source| Self::store_error("open", account.to_owned(), source))?,
        );
        entries.insert(account.to_owned(), entry.clone());
        Ok(entry)
    }

    // macOS keychain prompts per item and per app signature, so all secrets
    // share ONE blob entry there (one prompt per app version). Windows caps
    // credential blobs at 2560 bytes and neither Windows nor secret-service
    // prompt per item, so those platforms store one entry per secret.
    fn blob_mode() -> bool {
        cfg!(target_os = "macos")
    }

    fn store_error(
        operation: &'static str,
        reference: String,
        source: impl ToString,
    ) -> StorageError {
        StorageError::SecretStore {
            operation,
            reference,
            message: source.to_string(),
        }
    }

    fn blob_entry(&self) -> Result<std::sync::Arc<keyring::Entry>> {
        self.entry(KEYRING_BLOB_ACCOUNT)
    }

    fn secret_entry(&self, reference: &SecretReference) -> Result<std::sync::Arc<keyring::Entry>> {
        self.entry(&reference.keyring_account())
    }

    fn entry_put(&self, reference: &SecretReference, value: &SecretMaterial) -> Result<()> {
        self.secret_entry(reference)?
            .set_password(value.expose_secret())
            .map_err(|source| Self::store_error("write", reference.keyring_account(), source))
    }

    fn entry_get(&self, reference: &SecretReference) -> Result<Option<SecretMaterial>> {
        match self.secret_entry(reference)?.get_password() {
            Ok(value) => Ok(Some(SecretMaterial::new(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(source) => Err(Self::store_error(
                "read",
                reference.keyring_account(),
                source,
            )),
        }
    }

    fn entry_delete(&self, reference: &SecretReference) -> Result<()> {
        match self.secret_entry(reference)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(source) => Err(Self::store_error(
                "delete",
                reference.keyring_account(),
                source,
            )),
        }
    }
}

impl Default for OsKeyringSecretStore {
    fn default() -> Self {
        Self::new(KEYRING_SERVICE)
    }
}

impl MapBacked for OsKeyringSecretStore {
    fn load_map(&self) -> Result<BTreeMap<String, String>> {
        match self.blob_entry()?.get_password() {
            Ok(raw) => serde_json::from_str(&raw).map_err(|source| {
                Self::store_error("parse", KEYRING_BLOB_ACCOUNT.to_owned(), source)
            }),
            Err(keyring::Error::NoEntry) => Ok(BTreeMap::new()),
            Err(source) => Err(Self::store_error(
                "read",
                KEYRING_BLOB_ACCOUNT.to_owned(),
                source,
            )),
        }
    }

    fn save_map(&self, map: &BTreeMap<String, String>) -> Result<()> {
        if map.is_empty() {
            return match self.blob_entry()?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(source) => Err(Self::store_error(
                    "delete",
                    KEYRING_BLOB_ACCOUNT.to_owned(),
                    source,
                )),
            };
        }
        let raw = serde_json::to_string(map).map_err(|source| {
            Self::store_error("serialize", KEYRING_BLOB_ACCOUNT.to_owned(), source)
        })?;
        self.blob_entry()?
            .set_password(&raw)
            .map_err(|source| Self::store_error("write", KEYRING_BLOB_ACCOUNT.to_owned(), source))
    }
}

impl SecretStore for OsKeyringSecretStore {
    fn put(&self, reference: &SecretReference, value: &SecretMaterial) -> Result<()> {
        if Self::blob_mode() {
            self.map_put(reference, value)
        } else {
            self.entry_put(reference, value)
        }
    }

    fn get(&self, reference: &SecretReference) -> Result<Option<SecretMaterial>> {
        if Self::blob_mode() {
            self.map_get(reference)
        } else {
            self.entry_get(reference)
        }
    }

    fn delete(&self, reference: &SecretReference) -> Result<()> {
        if Self::blob_mode() {
            self.map_delete(reference)
        } else {
            self.entry_delete(reference)
        }
    }

    fn get_all(&self, references: &[SecretReference]) -> Result<Vec<Option<SecretMaterial>>> {
        if Self::blob_mode() {
            self.map_get_all(references)
        } else {
            references.iter().map(|r| self.entry_get(r)).collect()
        }
    }

    fn apply(&self, puts: &[ImportedSecret], deletes: &[SecretReference]) -> Result<()> {
        if Self::blob_mode() {
            self.map_apply(puts, deletes)
        } else {
            for secret in puts {
                self.entry_put(&secret.reference, &secret.value)?;
            }
            for reference in deletes {
                self.entry_delete(reference)?;
            }
            Ok(())
        }
    }
}

#[path = "passwords_file_store.rs"]
mod file_store;
use file_store::EncryptedFileSecretStore;

/// Store used by the app: encrypted file when CORREOMQTT_MASTER_PASSWORD is
/// set (headless fallback, Java UserInputKeyring counterpart), OS keyring
/// otherwise.
pub fn default_secret_store() -> Box<dyn SecretStore> {
    match std::env::var("CORREOMQTT_MASTER_PASSWORD") {
        Ok(password) if !password.trim().is_empty() => Box::new(EncryptedFileSecretStore::new(
            config_root().join("secrets.enc"),
            password,
        )),
        _ => Box::new(OsKeyringSecretStore::default()),
    }
}

#[cfg(test)]
#[path = "passwords_tests.rs"]
mod tests;
