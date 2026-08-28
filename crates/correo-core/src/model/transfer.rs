use std::path::Path;

use correo_storage::current::{
    read_message_export, write_message_export, Message as StoredMessage, MessageType,
    PublishStatus, Qos as StoredQos,
};

use crate::{
    AppModel, ConnectionSurface, Diagnostic, QosLevel, TransferFeedback, TransferOutcome,
    WorkflowFeedback, Workspace,
};

impl AppModel {
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
