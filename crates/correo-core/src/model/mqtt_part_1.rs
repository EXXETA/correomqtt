use correo_mqtt::{IncomingMessage, Qos, SessionState, TopicFilter, TopicName};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{
    AppModel, ConnectDisabledReason, ConnectionState, Diagnostic, MessageDiagnosticRow, MessageRow,
    MqttCommand, MqttEvent, MqttFailure, MqttOperation, PublishHistoryRow, QosLevel,
    SubscriptionRow, WorkbenchSnapshot, WorkflowFeedback,
};

const MAX_INCOMING_MESSAGES: usize = 1_000;
const MAX_PUBLISH_HISTORY_ROWS: usize = 500;

impl AppModel {
    pub(super) fn update_publish_topic(&mut self, topic: String) {
        self.snapshot.workbench.publish.topic = topic;
        self.refresh_publish_validation();
    }

    pub(super) fn update_publish_payload(&mut self, payload: String) {
        self.snapshot.workbench.publish.payload = payload;
        self.refresh_publish_validation();
    }

    pub(super) fn copy_publish_history_message_to_publish_form(&mut self, id: u32) {
        let Some(row) = self
            .snapshot
            .workbench
            .publish
            .history
            .iter()
            .find(|row| row.id == id)
            .cloned()
        else {
            return;
        };
        self.snapshot.workbench.publish.topic = row.topic;
        self.snapshot.workbench.publish.qos = row.qos;
        self.snapshot.workbench.publish.retained = row.retained;
        self.snapshot.workbench.publish.payload =
            String::from_utf8_lossy(&row.payload).into_owned();
        self.snapshot.workbench.publish.selected_history_id = Some(id);
        self.snapshot.workbench.narrow_tab = crate::WorkbenchTab::Publish;
        self.refresh_publish_validation();
    }

    pub(super) fn copy_incoming_message_to_publish_form(&mut self, id: u32) {
        let Some(message) = self
            .snapshot
            .workbench
            .messages
            .iter()
            .find(|message| message.id == id)
            .cloned()
        else {
            return;
        };
        self.snapshot.workbench.publish.topic = message.topic;
        self.snapshot.workbench.publish.qos = message.qos;
        self.snapshot.workbench.publish.retained = message.retained;
        self.snapshot.workbench.publish.payload =
            String::from_utf8_lossy(&message.payload).into_owned();
        self.snapshot.workbench.selected_message_id = Some(id);
        self.snapshot.workbench.narrow_tab = crate::WorkbenchTab::Publish;
        self.refresh_publish_validation();
    }

    pub(super) fn clear_publish_history(&mut self) {
        self.snapshot.workbench.publish.history.clear();
        self.snapshot.workbench.publish.selected_history_id = None;
    }

    pub(super) fn remove_publish_history_message(&mut self, id: u32) {
        self.snapshot
            .workbench
            .publish
            .history
            .retain(|message| message.id != id);
        if self.snapshot.workbench.publish.selected_history_id == Some(id) {
            self.snapshot.workbench.publish.selected_history_id = self
                .snapshot
                .workbench
                .publish
                .history
                .first()
                .map(|message| message.id);
        }
    }

    pub(super) fn clear_incoming_messages(&mut self) {
        self.snapshot.workbench.messages.clear();
        self.snapshot.workbench.selected_message_id = None;
        for subscription in &mut self.snapshot.workbench.subscribe.subscriptions {
            subscription.message_count = 0;
        }
    }

    pub(super) fn remove_incoming_message(&mut self, id: u32) {
        let Some(message) = self
            .snapshot
            .workbench
            .messages
            .iter()
            .find(|message| message.id == id)
            .cloned()
        else {
            return;
        };
        self.snapshot
            .workbench
            .messages
            .retain(|message| message.id != id);
        if self.snapshot.workbench.selected_message_id == Some(id) {
            self.snapshot.workbench.selected_message_id = self
                .snapshot
                .workbench
                .messages
                .first()
                .map(|message| message.id);
        }
        decrement_matching_subscriptions(&mut self.snapshot.workbench, &message.topic);
    }

    pub(super) fn update_publish_qos(&mut self, qos: QosLevel) {
        self.snapshot.workbench.publish.qos = qos;
        self.refresh_publish_validation();
    }

    pub(super) fn update_subscribe_topic(&mut self, topic: String) {
        self.snapshot.workbench.subscribe.topic = topic;
        self.refresh_subscribe_validation();
    }

    pub(super) fn update_subscribe_qos(&mut self, qos: QosLevel) {
        self.snapshot.workbench.subscribe.qos = qos;
        self.refresh_subscribe_validation();
    }

    pub(super) fn apply_mqtt_command(&mut self, command: MqttCommand) {
        match command {
            MqttCommand::Connect { options } => {
                self.mark_command_accepted(options.connection_id, MqttOperation::Connect);
            }
            MqttCommand::Reconnect { options } => {
                self.mark_reconnecting(options.connection_id, 1);
            }
            MqttCommand::Disconnect { connection_id } => {
                self.mark_command_accepted(connection_id, MqttOperation::Disconnect);
            }
            MqttCommand::Publish { connection_id, .. } => {
                self.mark_command_accepted(connection_id, MqttOperation::Publish);
            }
            MqttCommand::Subscribe {
                connection_id,
                subscription,
            } => {
                let _ = subscription;
                self.mark_command_accepted(connection_id, MqttOperation::Subscribe);
            }
            MqttCommand::Unsubscribe {
                connection_id,
                request,
            } => {
                let _ = request;
                self.mark_command_accepted(connection_id, MqttOperation::Unsubscribe);
            }
            MqttCommand::Shutdown => {}
        }
    }

    pub(super) fn apply_mqtt_event(&mut self, event: MqttEvent) {
        match event {
            MqttEvent::CommandAccepted {
                connection_id,
                operation,
            } => self.mark_command_accepted(connection_id, operation),
            MqttEvent::Connected { connection_id } => self.mark_connected(connection_id),
            MqttEvent::Disconnected { connection_id } => self.mark_disconnected(connection_id),
            MqttEvent::Reconnecting {
                connection_id,
                attempt,
            } => self.mark_reconnecting(connection_id, attempt),
            MqttEvent::StateChanged {
                connection_id,
                state,
            } => self.apply_session_state(connection_id, state),
            MqttEvent::IncomingMessage(message) => self.add_incoming_message(message),
            MqttEvent::Published {
                connection_id,
                topic,
                payload,
                qos,
                retain,
                diagnostics,
            } => {
                self.add_publish_success(
                    connection_id,
                    topic.as_str(),
                    payload,
                    qos_level(qos),
                    retain,
                    diagnostics,
                );
                self.push_diagnostic(Diagnostic::info(format!(
                    "MQTT publish completed for {} on {}.",
                    topic.as_str(),
                    connection_label(self, connection_id)
                )));
            }
            MqttEvent::Subscribed {
                connection_id,
                subscription,
            } => {
                self.add_subscription(
                    connection_id,
                    SubscriptionRow {
                        topic_filter: subscription.topic_filter.as_str().to_owned(),
                        qos: qos_level(subscription.qos),
                        message_count: 0,
                        active: true,
                        messages_visible: true,
                        selected: false,
                    },
                );
                self.set_subscribe_feedback(
                    connection_id,
                    WorkflowFeedback::info(format!(
                        "Subscribed to {}.",
                        subscription.topic_filter.as_str()
                    )),
                );
                self.push_diagnostic(Diagnostic::info(format!(
                    "MQTT subscribe completed for {} on {}.",
                    subscription.topic_filter.as_str(),
                    connection_label(self, connection_id)
                )));
            }
            MqttEvent::Unsubscribed {
                connection_id,
                request,
            } => {
                self.remove_subscription(connection_id, request.topic_filter.as_str());
                self.set_subscribe_feedback(
                    connection_id,
                    WorkflowFeedback::info(format!(
                        "Unsubscribed from {}.",
                        request.topic_filter.as_str()
                    )),
                );
                self.push_diagnostic(Diagnostic::info(format!(
                    "MQTT unsubscribe completed for {} on {}.",
                    request.topic_filter.as_str(),
                    connection_label(self, connection_id)
                )));
            }
            MqttEvent::Failure(failure) => self.apply_mqtt_failure(failure),
            MqttEvent::ShutdownComplete => {
                self.push_diagnostic(Diagnostic::info("MQTT service shutdown completed."));
            }
        }
    }

    fn mark_command_accepted(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        operation: MqttOperation,
    ) {
        match operation {
            MqttOperation::Connect => {
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Connecting,
                    Some(ConnectDisabledReason::Busy),
                    "connect command queued".to_owned(),
                );
            }
            MqttOperation::Disconnect => {
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Disconnected,
                    None,
                    "disconnect command queued".to_owned(),
                );
            }
            MqttOperation::Publish => {
                self.set_publish_feedback(
                    connection_id,
                    WorkflowFeedback::info("Publish accepted by MQTT service."),
                );
            }
            MqttOperation::Subscribe => {
                self.set_subscribe_feedback(
                    connection_id,
                    WorkflowFeedback::info("Subscribe accepted by MQTT service."),
                );
            }
            MqttOperation::Unsubscribe => {
                self.set_subscribe_feedback(
                    connection_id,
                    WorkflowFeedback::info("Unsubscribe accepted by MQTT service."),
                );
            }
            _ => {}
        }
    }

}

