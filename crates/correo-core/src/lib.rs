mod bootstrap;
mod broker;
mod commands;
mod history;
mod migration;
mod model;
mod mqtt;
mod runtime;
mod samples;
mod scripting;
mod scripting_mqtt;
#[cfg(test)]
mod scripting_tests;
mod settings_persistence;
mod surfaces;
mod types;

pub use commands::*;
pub use correo_diagnostics::{redact_sensitive, Diagnostic, DiagnosticSeverity};
pub use history::*;
pub use migration::*;
pub use model::AppModel;
pub use mqtt::*;
pub use runtime::{AppRuntime, PumpReport};
pub use samples::sample_snapshot;
pub use scripting::*;
pub use settings_persistence::*;
pub use surfaces::*;
use thiserror::Error;
pub use types::*;

use correo_mqtt::ConnectionId;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("connection is not open: {0}")]
    ConnectionNotOpen(ConnectionId),
}
pub use bootstrap::{
    startup_state_from_current, startup_state_from_current_with_plugins,
    startup_state_from_current_with_workbenches, startup_state_from_migration, StartupState,
};
pub use broker::*;
