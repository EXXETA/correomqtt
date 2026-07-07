use std::collections::HashSet;
use std::path::Path;

use correo_storage::current::{
    decrypt_connection_export, read_connection_export, ConnectionConfig, ConnectionExport,
    ConnectionImport, ImportedSecret, MqttVersion as StoredMqttVersion,
};

use crate::{
    AppModel, ConnectionImportSnapshot, ConnectionSummary, ConnectionSurface, Diagnostic,
    ImportPasswordState, TransferConnectionRow, TransferConnectionStatus, TransferFeedback,
    TransferFileSnapshot, TransferOutcome, TransferSection, TransferStep, Workspace,
};

impl AppModel {
    pub(super) fn import_connections(&mut self) {
        self.clear_pending_connection_import();
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Transfer;
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.active_step = TransferStep::ChooseFile;
        self.snapshot.transfer.import = ConnectionImportSnapshot::default();
        self.snapshot.transfer.selected_connections = 0;
        self.push_diagnostic(Diagnostic::info(
            "Connection import command queued for a .cqc file.",
        ));
    }

    pub(super) fn choose_connection_import_file(&mut self, path: &Path) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Transfer;
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.import.outcome = None;

        match read_connection_export(path) {
            Ok(ConnectionExport::Encrypted(export)) => {
                self.clear_pending_connection_import();
                self.snapshot.transfer.import.file = Some(file_snapshot(path, 0, true));
                self.snapshot.transfer.import.encrypted = true;
                self.snapshot.transfer.import.password_state = ImportPasswordState::Needed;
                self.snapshot.transfer.import.rows.clear();
                self.snapshot.transfer.selected_connections = 0;
                self.snapshot.transfer.import.warnings = export
                    .warnings
                    .into_iter()
                    .map(|warning| warning.message)
                    .collect();
                self.snapshot.transfer.import.feedback = None;
                self.snapshot.transfer.active_step = TransferStep::Password;
            }
            Ok(ConnectionExport::Plain(import)) => {
                let connection_count = import.connections.len();
                let rows = self.import_connection_rows(&import.connections);
                let warnings = import
                    .warnings
                    .iter()
                    .map(|warning| warning.message.clone())
                    .collect();
                self.stage_connection_import(import);
                self.snapshot.transfer.import.file =
                    Some(file_snapshot(path, connection_count, false));
                self.snapshot.transfer.import.encrypted = false;
                self.snapshot.transfer.import.password_state = ImportPasswordState::NotNeeded;
                self.snapshot.transfer.import.rows = rows;
                self.snapshot.transfer.selected_connections =
                    self.snapshot.transfer.import.selected_count();
                self.snapshot.transfer.import.warnings = warnings;
                self.snapshot.transfer.import.feedback = Some(TransferFeedback::info(
                    "Selected .cqc file is ready for review.",
                ));
                self.snapshot.transfer.active_step = TransferStep::Review;
            }
            Err(error) => {
                self.clear_pending_connection_import();
                self.snapshot.transfer.import.file = Some(file_snapshot(path, 0, false));
                self.snapshot.transfer.import.encrypted = false;
                self.snapshot.transfer.import.password_state = ImportPasswordState::NotNeeded;
                self.snapshot.transfer.import.rows.clear();
                self.snapshot.transfer.selected_connections = 0;
                self.snapshot.transfer.import.feedback = Some(TransferFeedback::error(format!(
                    "Could not read selected connection file: {error}",
                )));
                self.snapshot.transfer.active_step = TransferStep::ChooseFile;
            }
        }
    }

    pub(super) fn submit_connection_import_password(&mut self, password: &str) {
        self.snapshot.transfer.active_section = TransferSection::Import;
        let Some(path) = self
            .snapshot
            .transfer
            .import
            .file
            .as_ref()
            .map(|file| file.path_hint.clone())
        else {
            self.snapshot.transfer.import.feedback = Some(TransferFeedback::error(
                "Choose a connection file before entering a password.",
            ));
            self.snapshot.transfer.active_step = TransferStep::ChooseFile;
            return;
        };

        match read_connection_export(&path).and_then(|export| match export {
            ConnectionExport::Encrypted(export) => decrypt_connection_export(&export, password),
            ConnectionExport::Plain(import) => Ok(import),
        }) {
            Ok(import) => {
                let connection_count = import.connections.len();
                let rows = self.import_connection_rows(&import.connections);
                let warnings = import
                    .warnings
                    .iter()
                    .map(|warning| warning.message.clone())
                    .collect();
                self.stage_connection_import(import);
                self.snapshot.transfer.import.file =
                    Some(file_snapshot(Path::new(&path), connection_count, true));
                self.snapshot.transfer.import.rows = rows;
                self.snapshot.transfer.selected_connections =
                    self.snapshot.transfer.import.selected_count();
                self.snapshot.transfer.import.warnings = warnings;
                self.snapshot.transfer.import.password_state = ImportPasswordState::Accepted;
                self.snapshot.transfer.import.feedback =
                    Some(TransferFeedback::info("Encrypted .cqc file unlocked."));
                self.snapshot.transfer.active_step = TransferStep::Review;
            }
            Err(error) => {
                self.clear_pending_connection_import();
                self.snapshot.transfer.import.password_state =
                    ImportPasswordState::InvalidRecoverable;
                self.snapshot.transfer.selected_connections = 0;
                self.snapshot.transfer.import.feedback = Some(TransferFeedback::error(format!(
                    "Password did not unlock this .cqc file: {error}",
                )));
                self.snapshot.transfer.active_step = TransferStep::Password;
            }
        }
    }

    fn stage_connection_import(&mut self, import: ConnectionImport) {
        self.pending_connection_imports = import
            .connections
            .into_iter()
            .map(|connection| (connection.id.clone(), connection))
            .collect();
        self.pending_connection_import_secrets = import.secrets;
        self.pending_connection_import_persistence = None;
    }

    fn clear_pending_connection_import(&mut self) {
        self.pending_connection_imports.clear();
        self.pending_connection_import_secrets.clear();
        self.pending_connection_import_persistence = None;
    }

    pub(super) fn clear_connection_import_error(&mut self) {
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.import.password_state = ImportPasswordState::Needed;
        self.snapshot.transfer.import.feedback = None;
        self.snapshot.transfer.active_step = TransferStep::Password;
    }

    pub(super) fn select_import_step(&mut self, step: TransferStep) {
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.active_step = step;
    }

    pub(super) fn select_connection_import_row(&mut self, row_id: &str, selected: bool) {
        if let Some(row) = self
            .snapshot
            .transfer
            .import
            .rows
            .iter_mut()
            .find(|row| row.id == row_id)
        {
            row.selected = selected && row.status.importable();
        }
        self.snapshot.transfer.selected_connections =
            self.snapshot.transfer.import.selected_count();
    }

    pub(super) fn start_connection_import(&mut self) {
        self.snapshot.transfer.active_section = TransferSection::Import;
        let selected = self.snapshot.transfer.import.selected_count();
        self.snapshot.transfer.active_step = TransferStep::Complete;
        if selected == 0 {
            self.snapshot.transfer.import.outcome = Some(TransferOutcome::failure(
                "Import failed",
                "Select at least one connection before importing.",
            ));
            self.snapshot.transfer.import.feedback =
                Some(TransferFeedback::warning("No connections selected."));
            return;
        }

        let selected_connections = self.selected_connection_imports();
        if selected_connections.len() != selected {
            self.snapshot.transfer.import.outcome = Some(TransferOutcome::failure(
                "Import failed",
                "Selected import data is no longer available. Choose the .cqc file again.",
            ));
            self.snapshot.transfer.import.feedback =
                Some(TransferFeedback::error("Import data is incomplete."));
            return;
        }

        let selected_secrets = self.selected_connection_import_secrets(&selected_connections);
        for connection in selected_connections.iter().cloned() {
            self.add_imported_connection(connection, &selected_secrets);
        }
        self.pending_connection_import_persistence =
            Some((selected_connections.clone(), selected_secrets));
        self.pending_connection_imports.clear();
        self.pending_connection_import_secrets.clear();
        self.snapshot.transfer.import.outcome = Some(TransferOutcome::success(
            "Import complete",
            format!("{selected} connection profiles imported; secrets stay in keyring."),
        ));
        self.snapshot.transfer.import.feedback = None;
    }

    fn selected_connection_imports(&self) -> Vec<ConnectionConfig> {
        self.snapshot
            .transfer
            .import
            .rows
            .iter()
            .filter(|row| row.selected && row.status.importable())
            .filter_map(|row| self.pending_connection_imports.get(&row.id).cloned())
            .collect()
    }

    fn selected_connection_import_secrets(
        &self,
        connections: &[ConnectionConfig],
    ) -> Vec<ImportedSecret> {
        let ids: HashSet<&str> = connections
            .iter()
            .map(|connection| connection.id.as_str())
            .collect();
        self.pending_connection_import_secrets
            .iter()
            .filter(|secret| ids.contains(secret.reference.connection_id.as_str()))
            .cloned()
            .collect()
    }
}

fn file_snapshot(
    path: &Path,
    detected_connections: usize,
    encrypted: bool,
) -> TransferFileSnapshot {
    TransferFileSnapshot {
        display_name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("connections.cqc")
            .to_owned(),
        path_hint: path.display().to_string(),
        byte_size: std::fs::metadata(path)
            .map(|metadata| metadata.len() as usize)
            .unwrap_or_default(),
        detected_connections,
        encrypted,
    }
}

impl AppModel {
    fn import_connection_rows(
        &self,
        connections: &[ConnectionConfig],
    ) -> Vec<TransferConnectionRow> {
        let mut conflict_ids: Vec<&str> = self
            .storage_connection_ids
            .values()
            .map(String::as_str)
            .collect();
        // A storage id repeated within the same import file would collapse in
        // the pending map and yield inconsistent runtime profiles, so every
        // such duplicate is treated as a conflict too.
        let mut seen = HashSet::new();
        for connection in connections {
            if !seen.insert(connection.id.as_str()) {
                conflict_ids.push(connection.id.as_str());
            }
        }
        connections
            .iter()
            .map(|connection| import_row(connection, &self.snapshot.connections, &conflict_ids))
            .collect()
    }
}

fn import_row(
    connection: &ConnectionConfig,
    existing: &[ConnectionSummary],
    existing_storage_ids: &[&str],
) -> TransferConnectionRow {
    let existing_names: Vec<&str> = existing
        .iter()
        .map(|current| current.name.as_str())
        .collect();
    let status = import_status(
        &connection.name,
        &connection.id,
        &existing_names,
        existing_storage_ids,
    );
    TransferConnectionRow {
        id: connection.id.clone(),
        name: connection.name.clone(),
        endpoint: format!("{}:{}", connection.url, connection.port),
        mqtt_version: stored_mqtt_version(connection.mqtt_version),
        selected: status.importable(),
        status,
    }
}

// A conflict is either a same-name profile or the same persisted storage id.
// Comparing against the runtime summary UUID (as before) would miss id
// collisions and let an import silently overwrite an existing on-disk
// connection.
fn import_status(
    name: &str,
    storage_id: &str,
    existing_names: &[&str],
    existing_storage_ids: &[&str],
) -> TransferConnectionStatus {
    if existing_names.contains(&name) || existing_storage_ids.contains(&storage_id) {
        TransferConnectionStatus::Conflict
    } else {
        TransferConnectionStatus::New
    }
}

fn stored_mqtt_version(version: StoredMqttVersion) -> String {
    match version {
        StoredMqttVersion::Mqtt311 => "MQTT 3.1.1".to_owned(),
        StoredMqttVersion::Mqtt50 => "MQTT v5".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{import_status, TransferConnectionStatus};

    #[test]
    fn same_storage_id_is_a_conflict_even_with_a_different_name() {
        // The bug: comparing the import's storage id against runtime UUIDs
        // missed this and let the import silently overwrite the on-disk record.
        let status = import_status("Fresh Name", "storage-1", &["Other"], &["storage-1"]);
        assert_eq!(status, TransferConnectionStatus::Conflict);
    }

    #[test]
    fn same_name_is_a_conflict() {
        let status = import_status("Broker", "storage-9", &["Broker"], &["storage-1"]);
        assert_eq!(status, TransferConnectionStatus::Conflict);
    }

    #[test]
    fn distinct_name_and_storage_id_is_new() {
        let status = import_status("Broker", "storage-9", &["Other"], &["storage-1"]);
        assert_eq!(status, TransferConnectionStatus::New);
    }
}
