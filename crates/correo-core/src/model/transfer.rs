use std::path::Path;

use correo_storage::current::{
    decrypt_connection_export, read_connection_export, read_message_export, write_message_export,
    ConnectionConfig, ConnectionExport, Message as StoredMessage, MessageType,
    MqttVersion as StoredMqttVersion, PublishStatus, Qos as StoredQos,
};

use crate::{
    AppModel, ConnectionSummary, ConnectionSurface, Diagnostic, ExportPasswordConfirmation,
    ExportPathState, ImportPasswordState, QosLevel, TransferConnectionRow,
    TransferConnectionStatus, TransferFeedback, TransferFileSnapshot, TransferOutcome,
    TransferSection, TransferStep, WorkflowFeedback, Workspace,
};

impl AppModel {
    pub(super) fn import_connections(&mut self) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Transfer;
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.active_step = TransferStep::ChooseFile;
        self.push_diagnostic(Diagnostic::info(
            "Connection import command queued for a .cqc file.",
        ));
    }

    pub(super) fn open_connection_export(&mut self) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Transfer;
        self.snapshot.transfer.active_section = TransferSection::Export;
        self.snapshot.transfer.export.rows =
            self.snapshot.connections.iter().map(export_row).collect();
        self.snapshot.transfer.selected_connections =
            self.snapshot.transfer.export.selected_count();
        self.snapshot.transfer.export.outcome = None;
        self.snapshot.transfer.export.feedback = Some(TransferFeedback::info(
            "Connection export is ready. Plain exports omit sensitive auth values.",
        ));
        self.push_diagnostic(Diagnostic::info("Connection export command queued."));
    }

    pub(super) fn choose_connection_import_file(&mut self, path: &Path) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Transfer;
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.import.outcome = None;

        match read_connection_export(path) {
            Ok(ConnectionExport::Encrypted(export)) => {
                self.snapshot.transfer.import.file = Some(file_snapshot(path, 0, true));
                self.snapshot.transfer.import.encrypted = true;
                self.snapshot.transfer.import.password_state = ImportPasswordState::Needed;
                self.snapshot.transfer.import.rows.clear();
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
                let warnings = import
                    .warnings
                    .into_iter()
                    .map(|warning| warning.message)
                    .collect();
                self.snapshot.transfer.import.file =
                    Some(file_snapshot(path, connection_count, false));
                self.snapshot.transfer.import.encrypted = false;
                self.snapshot.transfer.import.password_state = ImportPasswordState::NotNeeded;
                self.snapshot.transfer.import.rows = import
                    .connections
                    .iter()
                    .map(|connection| import_row(connection, &self.snapshot.connections))
                    .collect();
                self.snapshot.transfer.import.warnings = warnings;
                self.snapshot.transfer.import.feedback = Some(TransferFeedback::info(
                    "Selected .cqc file is ready for review.",
                ));
                self.snapshot.transfer.active_step = TransferStep::Review;
            }
            Err(error) => {
                self.snapshot.transfer.import.file = Some(file_snapshot(path, 0, false));
                self.snapshot.transfer.import.encrypted = false;
                self.snapshot.transfer.import.password_state = ImportPasswordState::NotNeeded;
                self.snapshot.transfer.import.rows.clear();
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
                let warnings = import
                    .warnings
                    .into_iter()
                    .map(|warning| warning.message)
                    .collect();
                self.snapshot.transfer.import.file =
                    Some(file_snapshot(Path::new(&path), connection_count, true));
                self.snapshot.transfer.import.rows = import
                    .connections
                    .iter()
                    .map(|connection| import_row(connection, &self.snapshot.connections))
                    .collect();
                self.snapshot.transfer.import.warnings = warnings;
                self.snapshot.transfer.import.password_state = ImportPasswordState::Accepted;
                self.snapshot.transfer.import.feedback =
                    Some(TransferFeedback::info("Encrypted .cqc file unlocked."));
                self.snapshot.transfer.active_step = TransferStep::Review;
            }
            Err(error) => {
                self.snapshot.transfer.import.password_state =
                    ImportPasswordState::InvalidRecoverable;
                self.snapshot.transfer.import.feedback = Some(TransferFeedback::error(format!(
                    "Password did not unlock this .cqc file: {error}",
                )));
                self.snapshot.transfer.active_step = TransferStep::Password;
            }
        }
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
        } else {
            self.snapshot.transfer.import.outcome = Some(TransferOutcome::success(
                "Import complete",
                format!("{selected} connection profiles imported; secrets stay in keyring."),
            ));
            self.snapshot.transfer.import.feedback = None;
        }
    }

    pub(super) fn select_connection_export_row(&mut self, row_id: &str, selected: bool) {
        if let Some(row) = self
            .snapshot
            .transfer
            .export
            .rows
            .iter_mut()
            .find(|row| row.id == row_id)
        {
            row.selected = selected;
        }
        self.snapshot.transfer.selected_connections =
            self.snapshot.transfer.export.selected_count();
    }

    pub(super) fn set_connection_export_encrypted(&mut self, encrypted: bool) {
        self.snapshot.transfer.active_section = TransferSection::Export;
        self.snapshot.transfer.export.encrypted = encrypted;
        self.snapshot.transfer.encrypted_export = encrypted;
        self.snapshot.transfer.export.password_confirmation = if encrypted {
            ExportPasswordConfirmation::Needed
        } else {
            ExportPasswordConfirmation::NotRequired
        };
        self.snapshot.transfer.export.feedback = Some(if encrypted {
            TransferFeedback::info("Encrypted export will require password confirmation.")
        } else {
            TransferFeedback::warning("Plain export excludes sensitive auth values.")
        });
    }

    pub(super) fn update_connection_export_path(&mut self, path: String) {
        self.snapshot.transfer.active_section = TransferSection::Export;
        self.snapshot.transfer.export.path_state = export_path_state(&path);
        self.snapshot.transfer.export.output_path = path;
        self.snapshot.transfer.export.feedback =
            path_feedback(self.snapshot.transfer.export.path_state);
    }

    pub(super) fn start_connection_export(&mut self) {
        self.snapshot.transfer.active_section = TransferSection::Export;
        let selected = self.snapshot.transfer.export.selected_count();
        let path_state = self.snapshot.transfer.export.path_state;
        if selected == 0 {
            self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "Select at least one connection before exporting.",
            ));
            return;
        }
        if path_state == ExportPathState::InvalidPath {
            self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "Choose a writable target path before exporting.",
            ));
            return;
        }
        let detail = if self.snapshot.transfer.export.encrypted {
            format!("{selected} encrypted connection profiles exported.")
        } else {
            format!("{selected} plain profiles exported without sensitive auth values.")
        };
        self.snapshot.transfer.export.outcome =
            Some(TransferOutcome::success("Export complete", detail));
    }

    pub(super) fn import_messages(&mut self) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Workbench;
        self.snapshot.workbench.narrow_tab = crate::WorkbenchTab::Publish;
        self.snapshot.workbench.publish.feedback = Some(WorkflowFeedback::info(
            "Choose a .cqm message file to load into the publish editor.",
        ));
        self.snapshot.transfer.messages.feedback = Some(TransferFeedback::info(
            "Message import command queued for a .cqm file.",
        ));
        self.push_diagnostic(Diagnostic::info("Message import command queued."));
    }

    pub(super) fn import_messages_from_path(&mut self, path: &Path) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Workbench;
        self.snapshot.workbench.narrow_tab = crate::WorkbenchTab::Publish;

        match read_message_export(path) {
            Ok(message) => {
                self.snapshot.workbench.publish.topic = message.topic;
                self.snapshot.workbench.publish.payload = message.payload.unwrap_or_default();
                self.snapshot.workbench.publish.retained = message.retained;
                if let Some(qos) = message.qos {
                    self.snapshot.workbench.publish.qos = stored_qos(qos);
                }
                self.refresh_publish_validation();
                self.snapshot.workbench.publish.feedback = Some(WorkflowFeedback::info(
                    "Loaded .cqm message into the publish editor.",
                ));
                self.snapshot.transfer.messages.feedback = Some(TransferFeedback::info(
                    "Loaded .cqm message into the publish editor.",
                ));
                self.push_diagnostic(Diagnostic::info("Message import completed."));
            }
            Err(error) => {
                self.snapshot.workbench.publish.feedback = Some(WorkflowFeedback::error(format!(
                    "Could not load .cqm message: {error}"
                )));
                self.snapshot.transfer.messages.feedback = Some(TransferFeedback::error(
                    "Could not load selected .cqm message file.",
                ));
                self.push_diagnostic(Diagnostic::warning(
                    "Message import failed for selected .cqm file.",
                ));
            }
        }
    }

    pub(super) fn export_messages(&mut self) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Workbench;
        let count = self.snapshot.transfer.messages.selected_messages;
        self.snapshot.transfer.messages.outcome = Some(TransferOutcome::success(
            "Message export ready",
            format!("{count} message snapshots queued for .cqm export."),
        ));
        self.push_diagnostic(Diagnostic::info("Message export command queued."));
    }

    pub(super) fn export_publish_history_message(&mut self, message_id: u32) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Workbench;
        self.snapshot.workbench.narrow_tab = crate::WorkbenchTab::Publish;
        let topic = self
            .snapshot
            .workbench
            .publish
            .history
            .iter()
            .find(|row| row.id == message_id)
            .map(|row| row.topic.clone())
            .unwrap_or_else(|| "selected topic".to_owned());
        self.snapshot.workbench.publish.feedback = Some(WorkflowFeedback::info(format!(
            "Queued outgoing message on {topic} for .cqm export."
        )));
        self.push_diagnostic(Diagnostic::info("Outgoing message export command queued."));
    }

    pub(super) fn export_publish_history_message_to_path(&mut self, message_id: u32, path: &Path) {
        let Some(row) = self
            .snapshot
            .workbench
            .publish
            .history
            .iter()
            .find(|row| row.id == message_id)
        else {
            self.snapshot.workbench.publish.feedback = Some(WorkflowFeedback::error(
                "Could not export outgoing message: message was not found.",
            ));
            return;
        };
        let message = StoredMessage {
            topic: row.topic.clone(),
            payload: Some(String::from_utf8_lossy(&row.payload).into_owned()),
            retained: row.retained,
            qos: Some(stored_qos_level(row.qos)),
            date_time: Some(row.timestamp.clone()),
            message_id: None,
            message_type: Some(MessageType::Outgoing),
            publish_status: Some(PublishStatus::Succeeded),
        };
        match write_message_export(path, &message) {
            Ok(()) => {
                self.snapshot.workbench.publish.feedback = Some(WorkflowFeedback::info(
                    "Saved outgoing message to .cqm file.",
                ));
                self.push_diagnostic(Diagnostic::info("Outgoing message export completed."));
            }
            Err(error) => {
                self.snapshot.workbench.publish.feedback = Some(WorkflowFeedback::error(format!(
                    "Could not save outgoing message: {error}"
                )));
                self.push_diagnostic(Diagnostic::warning(
                    "Outgoing message export failed for selected .cqm file.",
                ));
            }
        }
    }

    pub(super) fn export_incoming_message(&mut self, message_id: u32) {
        self.snapshot.active_workspace = Workspace::Connections;
        self.snapshot.connection_surface = ConnectionSurface::Workbench;
        self.snapshot.workbench.narrow_tab = crate::WorkbenchTab::Subscribe;
        self.snapshot.workbench.selected_message_id = Some(message_id);
        self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(
            "Queued selected incoming message for .cqm export.",
        ));
        self.push_diagnostic(Diagnostic::info("Incoming message export command queued."));
    }

    pub(super) fn export_incoming_message_to_path(&mut self, message_id: u32, path: &Path) {
        let Some(row) = self
            .snapshot
            .workbench
            .messages
            .iter()
            .find(|row| row.id == message_id)
        else {
            self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::error(
                "Could not export incoming message: message was not found.",
            ));
            return;
        };
        self.snapshot.workbench.selected_message_id = Some(message_id);
        let message = StoredMessage {
            topic: row.topic.clone(),
            payload: Some(String::from_utf8_lossy(&row.payload).into_owned()),
            retained: row.retained,
            qos: Some(stored_qos_level(row.qos)),
            date_time: Some(row.timestamp.clone()),
            message_id: None,
            message_type: Some(MessageType::Incoming),
            publish_status: None,
        };
        match write_message_export(path, &message) {
            Ok(()) => {
                self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(
                    "Saved incoming message to .cqm file.",
                ));
                self.push_diagnostic(Diagnostic::info("Incoming message export completed."));
            }
            Err(error) => {
                self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::error(
                    format!("Could not save incoming message: {error}"),
                ));
                self.push_diagnostic(Diagnostic::warning(
                    "Incoming message export failed for selected .cqm file.",
                ));
            }
        }
    }
}

fn stored_qos(qos: StoredQos) -> QosLevel {
    match qos {
        StoredQos::AtMostOnce => QosLevel::Zero,
        StoredQos::AtLeastOnce => QosLevel::One,
        StoredQos::ExactlyOnce => QosLevel::Two,
    }
}

fn stored_qos_level(qos: QosLevel) -> StoredQos {
    match qos {
        QosLevel::Zero => StoredQos::AtMostOnce,
        QosLevel::One => StoredQos::AtLeastOnce,
        QosLevel::Two => StoredQos::ExactlyOnce,
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

fn import_row(
    connection: &ConnectionConfig,
    existing: &[ConnectionSummary],
) -> TransferConnectionRow {
    let status = if existing
        .iter()
        .any(|current| current.name == connection.name || current.id.to_string() == connection.id)
    {
        TransferConnectionStatus::Conflict
    } else {
        TransferConnectionStatus::New
    };
    TransferConnectionRow {
        id: connection.id.clone(),
        name: connection.name.clone(),
        endpoint: format!("{}:{}", connection.url, connection.port),
        mqtt_version: stored_mqtt_version(connection.mqtt_version),
        selected: status.importable(),
        status,
    }
}

fn stored_mqtt_version(version: StoredMqttVersion) -> String {
    match version {
        StoredMqttVersion::Mqtt311 => "MQTT 3.1.1".to_owned(),
        StoredMqttVersion::Mqtt50 => "MQTT v5".to_owned(),
    }
}

fn export_row(connection: &ConnectionSummary) -> TransferConnectionRow {
    TransferConnectionRow {
        id: connection.id.to_string(),
        name: connection.name.clone(),
        endpoint: connection.endpoint.clone(),
        mqtt_version: connection.mqtt_version.clone(),
        selected: false,
        status: TransferConnectionStatus::Exportable,
    }
}

fn export_path_state(path: &str) -> ExportPathState {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        ExportPathState::InvalidPath
    } else if !trimmed.ends_with(".cqc") {
        ExportPathState::MissingExtension
    } else {
        ExportPathState::Ready
    }
}

fn path_feedback(state: ExportPathState) -> Option<TransferFeedback> {
    match state {
        ExportPathState::Ready => None,
        ExportPathState::MissingExtension => Some(TransferFeedback::warning(
            "The target file should end with .cqc.",
        )),
        ExportPathState::InvalidPath => Some(TransferFeedback::error(
            "Choose a writable target path before exporting.",
        )),
    }
}
