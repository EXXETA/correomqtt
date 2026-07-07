use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("failed to read {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse JSON from {path}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to create directory {path}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write {path}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to rename {from} to {to}")]
    Rename {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to copy {from} to {to}")]
    Copy {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to delete {path}")]
    Delete {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("script filename must be a safe relative .js path: {0}")]
    InvalidScriptFileName(String),
    #[error("script already exists: {0}")]
    ScriptAlreadyExists(String),
    #[error("script not found: {0}")]
    ScriptNotFound(String),
    #[error("script filename is unchanged: {0}")]
    ScriptNameUnchanged(String),
    #[error("unsupported legacy password encryption type: {0}")]
    UnsupportedPasswordEncryption(String),
    #[error("invalid legacy password payload: {0}")]
    InvalidPasswordPayload(&'static str),
    #[error("legacy password decryption failed")]
    PasswordDecryption,
    #[error("unsupported connection export encryption type: {0}")]
    UnsupportedConnectionExportEncryption(String),
    #[error("invalid connection export payload: {0}")]
    InvalidConnectionExportPayload(&'static str),
    #[error("connection export decryption failed")]
    ConnectionExportDecryption,
    #[error("failed to parse connection export JSON")]
    ConnectionExportJson {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialize connection export JSON")]
    ConnectionExportJsonSerialize {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to parse message export JSON")]
    MessageExportJson {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialize message export JSON")]
    MessageExportJsonSerialize {
        #[source]
        source: serde_json::Error,
    },
    #[error("legacy field {field} is required for {record}")]
    MissingField {
        record: &'static str,
        field: &'static str,
    },
    #[error("migration rollback safety check failed: {reason}")]
    MigrationRollbackSafety { reason: String },
    #[error("failed to {operation} secret {reference} in OS keyring: {message}")]
    SecretStore {
        operation: &'static str,
        reference: String,
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, StorageError>;

pub(crate) fn read_to_string(path: impl Into<PathBuf>) -> Result<String> {
    let path = path.into();
    std::fs::read_to_string(&path).map_err(|source| StorageError::Read { path, source })
}

pub(crate) fn read_json<T: serde::de::DeserializeOwned>(path: impl Into<PathBuf>) -> Result<T> {
    let path = path.into();
    let text = std::fs::read_to_string(&path).map_err(|source| StorageError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| StorageError::Json { path, source })
}
