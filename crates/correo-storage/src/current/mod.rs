mod config;
mod history;
mod hooks;
mod import_export;
mod message;
mod passwords;
mod scripting;

pub use config::*;
pub use history::*;
pub use hooks::*;
pub use import_export::*;
pub use message::*;
pub use passwords::*;
pub use scripting::*;

pub fn config_root() -> std::path::PathBuf {
    if let Some(path) = std::env::var_os("CORREOMQTT_CONFIG_DIR") {
        if !path.is_empty() {
            return path.into();
        }
    }
    directories::ProjectDirs::from("org", "CorreoMQTT", "CorreoMQTT")
        .map(|project_dirs| project_dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| ".correomqtt".into())
}
