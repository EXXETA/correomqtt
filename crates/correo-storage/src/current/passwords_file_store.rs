use std::collections::BTreeMap;

use super::{ImportedSecret, MapBacked, SecretMaterial, SecretReference, SecretStore};
use crate::{Result, StorageError};

/// Encrypted-file fallback for machines without a usable OS keyring
/// (e.g. headless Linux). Activated via CORREOMQTT_MASTER_PASSWORD; stores
/// the account→secret map AES-256-GCM encrypted with a PBKDF2-derived key.
pub struct EncryptedFileSecretStore {
    path: std::path::PathBuf,
    master_password: SecretMaterial,
}

const FILE_KDF_ITERATIONS: u32 = 100_000;
const SALT_BYTES: usize = 16;
const NONCE_BYTES: usize = 12;

#[derive(serde::Serialize, serde::Deserialize)]
struct EncryptedSecretsFile {
    salt: String,
    nonce: String,
    ciphertext: String,
}

impl EncryptedFileSecretStore {
    pub fn new(path: impl Into<std::path::PathBuf>, master_password: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            master_password: SecretMaterial::new(master_password),
        }
    }

    fn derive_key(&self, salt: &[u8]) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2::pbkdf2_hmac::<sha2::Sha256>(
            self.master_password.expose_secret().as_bytes(),
            salt,
            FILE_KDF_ITERATIONS,
            &mut key,
        );
        key
    }

    fn file_error(&self, operation: &'static str, message: impl ToString) -> StorageError {
        StorageError::SecretStore {
            operation,
            reference: self.path.display().to_string(),
            message: message.to_string(),
        }
    }
}

impl MapBacked for EncryptedFileSecretStore {
    fn load_map(&self) -> Result<BTreeMap<String, String>> {
        use aes_gcm::aead::Aead;
        use aes_gcm::KeyInit;
        use base64::Engine;

        if !self.path.exists() {
            return Ok(BTreeMap::new());
        }
        let raw =
            std::fs::read_to_string(&self.path).map_err(|error| self.file_error("read", error))?;
        let file: EncryptedSecretsFile =
            serde_json::from_str(&raw).map_err(|error| self.file_error("parse", error))?;
        let engine = base64::engine::general_purpose::STANDARD;
        let salt = engine
            .decode(&file.salt)
            .map_err(|error| self.file_error("parse", error))?;
        let nonce = engine
            .decode(&file.nonce)
            .map_err(|error| self.file_error("parse", error))?;
        // AES-256-GCM nonces are 12 bytes; Nonce::from_slice panics otherwise,
        // so a corrupt/truncated secrets.enc must fail as a recoverable error.
        if nonce.len() != NONCE_BYTES {
            return Err(self.file_error("parse", "AES-GCM nonce must be 12 bytes"));
        }
        let ciphertext = engine
            .decode(&file.ciphertext)
            .map_err(|error| self.file_error("parse", error))?;
        let key = self.derive_key(&salt);
        let cipher = aes_gcm::Aes256Gcm::new_from_slice(&key)
            .map_err(|error| self.file_error("decrypt", error))?;
        let plaintext = cipher
            .decrypt(aes_gcm::Nonce::from_slice(&nonce), ciphertext.as_slice())
            .map_err(|_| StorageError::PasswordDecryption)?;
        serde_json::from_slice(&plaintext).map_err(|error| self.file_error("parse", error))
    }

    fn save_map(&self, map: &BTreeMap<String, String>) -> Result<()> {
        use aes_gcm::aead::Aead;
        use aes_gcm::KeyInit;
        use base64::Engine;
        use rand::RngCore;

        if map.is_empty() {
            return match std::fs::remove_file(&self.path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(self.file_error("delete", error)),
            };
        }
        let plaintext =
            serde_json::to_vec(map).map_err(|error| self.file_error("serialize", error))?;
        let mut salt = [0u8; SALT_BYTES];
        let mut nonce = [0u8; NONCE_BYTES];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce);
        let key = self.derive_key(&salt);
        let cipher = aes_gcm::Aes256Gcm::new_from_slice(&key)
            .map_err(|error| self.file_error("encrypt", error))?;
        let ciphertext = cipher
            .encrypt(aes_gcm::Nonce::from_slice(&nonce), plaintext.as_slice())
            .map_err(|error| self.file_error("encrypt", error))?;
        let engine = base64::engine::general_purpose::STANDARD;
        let file = EncryptedSecretsFile {
            salt: engine.encode(salt),
            nonce: engine.encode(nonce),
            ciphertext: engine.encode(ciphertext),
        };
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| self.file_error("write", error))?;
        }
        let raw =
            serde_json::to_string(&file).map_err(|error| self.file_error("serialize", error))?;
        write_file_atomic(&self.path, raw.as_bytes()).map_err(|error| self.file_error("write", error))
    }
}

fn write_file_atomic(path: &std::path::Path, content: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&temporary, content)?;
    if cfg!(windows) && path.exists() {
        std::fs::remove_file(path)?;
    }
    std::fs::rename(temporary, path)
}

impl SecretStore for EncryptedFileSecretStore {
    fn put(&self, reference: &SecretReference, value: &SecretMaterial) -> Result<()> {
        self.map_put(reference, value)
    }

    fn get(&self, reference: &SecretReference) -> Result<Option<SecretMaterial>> {
        self.map_get(reference)
    }

    fn delete(&self, reference: &SecretReference) -> Result<()> {
        self.map_delete(reference)
    }

    fn get_all(&self, references: &[SecretReference]) -> Result<Vec<Option<SecretMaterial>>> {
        self.map_get_all(references)
    }

    fn apply(&self, puts: &[ImportedSecret], deletes: &[SecretReference]) -> Result<()> {
        self.map_apply(puts, deletes)
    }
}
