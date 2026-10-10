use crate::error::{read_json, Result, StorageError};
use aes::Aes128;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use serde::Deserialize;
use sha2::{Sha256, Sha512};
use std::collections::BTreeMap;
use std::path::Path;
use zeroize::Zeroize;

const AES_GCM: &str = "AES/GCM/NoPadding";
const AES_CBC: &str = "AES/CBC/PKCS5Padding";
const GCM_IV_BYTES: usize = 12;
const GCM_SALT_BYTES: usize = 16;

pub type LegacyPasswordMap = BTreeMap<String, String>;

/// Master password the Java app stored in the OS keyring (macOS Keychain,
/// libsecret) under service "CorreoMQTT". Lets migration import secrets
/// without prompting; users of Java's UserInputKeyring have no such entry
/// and fall back to the manual prompt.
pub fn os_keyring_master_password() -> Option<String> {
    let entry = keyring::Entry::new("CorreoMQTT", "CorreoMQTT_MasterPassword").ok()?;
    match entry.get_password() {
        Ok(password) if !password.trim().is_empty() => Some(password),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyPasswords {
    #[serde(default)]
    pub encryption_type: Option<String>,
    #[serde(default)]
    pub salt: Option<String>,
    #[serde(default)]
    pub passwords: Option<String>,
}

impl LegacyPasswords {
    pub fn read_from(path: impl AsRef<Path>) -> Result<Self> {
        read_json(path.as_ref().to_path_buf())
    }

    pub fn decrypt(&self, master_password: &str) -> Result<LegacyPasswordMap> {
        let Some(passwords) = self.passwords.as_deref() else {
            return Ok(LegacyPasswordMap::new());
        };
        if passwords.is_empty() {
            return Ok(LegacyPasswordMap::new());
        }

        let plaintext = match self.encryption_type.as_deref() {
            Some(AES_GCM) => decrypt_gcm(passwords, master_password)?,
            Some(AES_CBC) | None => {
                let salt = self
                    .salt
                    .as_deref()
                    .ok_or(StorageError::InvalidPasswordPayload("missing CBC salt"))?;
                decrypt_cbc(salt, passwords, master_password)?
            }
            Some(other) => {
                return Err(StorageError::UnsupportedPasswordEncryption(
                    other.to_owned(),
                ));
            }
        };

        serde_json::from_str(&plaintext).map_err(|_| StorageError::PasswordDecryption)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn key(self, connection_id: &str) -> String {
        format!("{}_{}", connection_id, self.label())
    }
}

fn decrypt_gcm(encrypted_data: &str, master_password: &str) -> Result<String> {
    let payload = STANDARD
        .decode(encrypted_data)
        .map_err(|_| StorageError::InvalidPasswordPayload("GCM payload is not base64"))?;
    if payload.len() <= GCM_IV_BYTES + GCM_SALT_BYTES {
        return Err(StorageError::InvalidPasswordPayload(
            "GCM payload is too short",
        ));
    }

    let (iv, rest) = payload.split_at(GCM_IV_BYTES);
    let (salt, ciphertext) = rest.split_at(GCM_SALT_BYTES);
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(master_password.as_bytes(), salt, 65_536, &mut key);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| StorageError::PasswordDecryption)?;
    let decrypted = cipher
        .decrypt(Nonce::from_slice(iv), ciphertext)
        .map_err(|_| StorageError::PasswordDecryption)?;
    key.zeroize();
    String::from_utf8(decrypted).map_err(|_| StorageError::PasswordDecryption)
}

fn decrypt_cbc(salt: &str, encrypted_data: &str, master_password: &str) -> Result<String> {
    let mut parts = encrypted_data.split(':');
    let iv = parts
        .next()
        .ok_or(StorageError::InvalidPasswordPayload("missing CBC IV"))?;
    let ciphertext = parts.next().ok_or(StorageError::InvalidPasswordPayload(
        "missing CBC ciphertext",
    ))?;
    if parts.next().is_some() {
        return Err(StorageError::InvalidPasswordPayload("too many CBC fields"));
    }

    let iv = STANDARD
        .decode(iv)
        .map_err(|_| StorageError::InvalidPasswordPayload("CBC IV is not base64"))?;
    let ciphertext = STANDARD
        .decode(ciphertext)
        .map_err(|_| StorageError::InvalidPasswordPayload("CBC ciphertext is not base64"))?;
    let mut key = [0u8; 16];
    pbkdf2_hmac::<Sha512>(
        master_password.as_bytes(),
        salt.as_bytes(),
        40_000,
        &mut key,
    );
    let decrypted = cbc::Decryptor::<Aes128>::new_from_slices(&key, &iv)
        .map_err(|_| StorageError::PasswordDecryption)?
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|_| StorageError::PasswordDecryption)?;
    key.zeroize();
    String::from_utf8(decrypted).map_err(|_| StorageError::PasswordDecryption)
}
