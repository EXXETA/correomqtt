use correo_storage::current::{
    write_encrypted_connection_export, write_plain_connection_export, ConnectionImport,
    ImportedSecret, SecretKind, SecretMaterial, SecretReference,
};

use crate::{
    AppModel, ConnectionSummary, ConnectionSurface, Diagnostic, ExportPasswordConfirmation,
    ExportPathState, SecretInput, TransferConnectionRow, TransferConnectionStatus,
    TransferFeedback, TransferOutcome, TransferSection, Workspace,
};

impl AppModel {
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

    pub(super) fn start_connection_export(&mut self, password: &SecretInput) {
        self.snapshot.transfer.active_section = TransferSection::Export;
        let selected = self.snapshot.transfer.export.selected_count();
        let path_state = self.snapshot.transfer.export.path_state;
        self.snapshot.transfer.export.outcome = None;
        if selected == 0 {
            self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "Select at least one connection before exporting.",
            ));
            self.snapshot.transfer.export.feedback =
                Some(TransferFeedback::warning("No connections selected."));
            return;
        }
        if path_state == ExportPathState::InvalidPath {
            self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "Choose a writable target path before exporting.",
            ));
            self.snapshot.transfer.export.feedback = Some(TransferFeedback::error(
                "Export target path is not writable.",
            ));
            return;
        }
        if self.snapshot.transfer.export.encrypted && password.is_empty() {
            self.snapshot.transfer.export.password_confirmation =
                ExportPasswordConfirmation::Needed;
            self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "Enter an export password before writing an encrypted .cqc file.",
            ));
            self.snapshot.transfer.export.feedback =
                Some(TransferFeedback::error("Export password is required."));
            return;
        }

        let Some(import) = self.connection_export_payload(self.snapshot.transfer.export.encrypted)
        else {
            self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "Selected connection profiles are no longer available.",
            ));
            self.snapshot.transfer.export.feedback = Some(TransferFeedback::error(
                "Selected connections could not be exported.",
            ));
            return;
        };
        let output_path = self.snapshot.transfer.export.output_path.clone();
        let result = if self.snapshot.transfer.export.encrypted {
            write_encrypted_connection_export(&output_path, &import, password.expose_for_ui())
        } else {
            write_plain_connection_export(&output_path, &import)
        };

        match result {
            Ok(()) => {
                let detail = if self.snapshot.transfer.export.encrypted {
                    self.snapshot.transfer.export.password_confirmation =
                        ExportPasswordConfirmation::Confirmed;
                    format!("{selected} encrypted connection profiles exported.")
                } else {
                    format!("{selected} plain profiles exported without sensitive auth values.")
                };
                self.snapshot.transfer.export.outcome =
                    Some(TransferOutcome::success("Export complete", detail));
                self.snapshot.transfer.export.feedback = None;
                self.push_diagnostic(Diagnostic::info("Connection export completed."));
            }
            Err(error) => {
                self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                    "Export failed",
                    "Could not write selected connection profiles.",
                ));
                self.snapshot.transfer.export.feedback = Some(TransferFeedback::error(format!(
                    "Connection export failed: {error}"
                )));
                self.push_diagnostic(Diagnostic::warning("Connection export failed."));
            }
        }
    }

    fn connection_export_payload(&self, include_secrets: bool) -> Option<ConnectionImport> {
        let mut connections = Vec::new();
        let mut secrets = Vec::new();

        for row in self
            .snapshot
            .transfer
            .export
            .rows
            .iter()
            .filter(|row| row.selected)
        {
            let connection = self
                .snapshot
                .connections
                .iter()
                .find(|connection| connection.id.to_string() == row.id)?;
            let settings = self.connection_settings_for(connection.id)?.clone();
            let storage_id = self.storage_connection_id(connection.id);
            if include_secrets {
                append_export_secret(
                    &mut secrets,
                    &storage_id,
                    SecretKind::Password,
                    &settings.password,
                );
                append_export_secret(
                    &mut secrets,
                    &storage_id,
                    SecretKind::SslKeystorePassword,
                    &settings.tls_keystore_password,
                );
                append_export_secret(
                    &mut secrets,
                    &storage_id,
                    SecretKind::AuthPassword,
                    &settings.ssh_password,
                );
            }
            connections.push(crate::settings_persistence::storage_connection(
                storage_id, settings,
            ));
        }

        Some(ConnectionImport {
            connections,
            secrets,
            warnings: Vec::new(),
        })
    }
}

fn append_export_secret(
    secrets: &mut Vec<ImportedSecret>,
    connection_id: &str,
    kind: SecretKind,
    value: &SecretInput,
) {
    if let Some(value) = value.expose_non_empty() {
        secrets.push(ImportedSecret {
            reference: SecretReference {
                connection_id: connection_id.to_owned(),
                kind,
            },
            value: SecretMaterial::new(value.to_owned()),
        });
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
