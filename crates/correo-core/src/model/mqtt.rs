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

impl AppModel {
    fn apply_session_state(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        state: SessionState,
    ) {
        match state {
            SessionState::Disconnected => self.mark_disconnected(connection_id),
            SessionState::Connecting => self.update_connection_state(
                connection_id,
                ConnectionState::Connecting,
                Some(ConnectDisabledReason::Busy),
                "connecting".to_owned(),
            ),
            SessionState::Connected => self.mark_connected(connection_id),
            SessionState::Disconnecting => self.update_connection_state(
                connection_id,
                ConnectionState::Disconnected,
                Some(ConnectDisabledReason::Busy),
                "disconnecting".to_owned(),
            ),
            SessionState::Reconnecting { attempt } => {
                self.mark_reconnecting(connection_id, attempt)
            }
            SessionState::Faulted { error } => {
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Error,
                    None,
                    "MQTT session faulted".to_owned(),
                );
                self.push_diagnostic(Diagnostic::error(error.message));
            }
        }
    }

    fn mark_connected(&mut self, connection_id: correo_mqtt::ConnectionId) {
        self.snapshot.active_connection = Some(connection_id);
        self.update_connection_state(
            connection_id,
            ConnectionState::Connected,
            Some(ConnectDisabledReason::AlreadyConnected),
            "connected".to_owned(),
        );
    }

    fn mark_disconnected(&mut self, connection_id: correo_mqtt::ConnectionId) {
        if self.snapshot.active_connection == Some(connection_id) {
            self.snapshot.active_connection = None;
        }
        self.update_connection_state(
            connection_id,
            ConnectionState::Disconnected,
            None,
            "disconnected".to_owned(),
        );
    }

    fn mark_reconnecting(&mut self, connection_id: correo_mqtt::ConnectionId, attempt: u32) {
        self.update_connection_state(
            connection_id,
            ConnectionState::Reconnecting,
            Some(ConnectDisabledReason::Busy),
            format!("reconnect attempt {attempt}"),
        );
        self.workbench_for_connection_mut(connection_id)
            .reconnect_status = format!("Reconnect attempt {attempt}");
        self.mark_workbench_dirty(connection_id);
    }

    fn add_incoming_message(&mut self, message: IncomingMessage) {
        let topic = message.topic.as_str().to_owned();
        let connection_id = message.connection_id;
        let row = MessageRow {
            id: self
                .workbench_for_connection(connection_id)
                .map(next_message_id)
                .unwrap_or(1),
            topic: topic.clone(),
            timestamp: current_timestamp(),
            qos: qos_level(message.qos),
            retained: message.retain,
            payload: message.payload.clone(),
            payload_preview: payload_preview(&message.payload),
            byte_size: message.payload.len(),
            badges: incoming_badges(&message),
            diagnostics: Vec::new(),
            formatted_detail: None,
        };
        let message_id = row.id;
        {
            let workbench = self.workbench_for_connection_mut(connection_id);
            workbench.messages.insert(0, row);
            if workbench.selected_message_id.is_none() {
                workbench.selected_message_id = Some(message_id);
            }
        }
        self.increment_matching_subscriptions(connection_id, &topic);
        self.update_recent_message_count(connection_id);
        self.prune_incoming_messages(connection_id);
        self.mark_workbench_dirty(connection_id);
    }

    fn increment_matching_subscriptions(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        topic: &str,
    ) {
        for subscription in &mut self
            .workbench_for_connection_mut(connection_id)
            .subscribe
            .subscriptions
        {
            if subscription.active && topic_matches_filter(topic, &subscription.topic_filter) {
                subscription.message_count = subscription.message_count.saturating_add(1);
            }
        }
    }

    fn prune_incoming_messages(&mut self, connection_id: correo_mqtt::ConnectionId) {
        let removed_topics = {
            let workbench = self.workbench_for_connection_mut(connection_id);
            if workbench.messages.len() <= MAX_INCOMING_MESSAGES {
                return;
            }
            workbench
                .messages
                .split_off(MAX_INCOMING_MESSAGES)
                .into_iter()
                .map(|message| message.topic)
                .collect::<Vec<_>>()
        };

        for topic in removed_topics {
            decrement_matching_subscriptions(
                self.workbench_for_connection_mut(connection_id),
                &topic,
            );
        }
    }

    fn update_recent_message_count(&mut self, connection_id: correo_mqtt::ConnectionId) {
        if let Some(index) = self.connection_index(connection_id) {
            self.snapshot.connections[index].recent_messages = self.snapshot.connections[index]
                .recent_messages
                .saturating_add(1);
            self.snapshot.connections[index].last_activity = "message received".to_owned();
        }
    }

    fn apply_mqtt_failure(&mut self, failure: MqttFailure) {
        let message = format!(
            "MQTT {} failed: {}",
            failure.operation.label(),
            failure.report.message
        );
        if let Some(connection_id) = failure.connection_id {
            match failure.operation {
                MqttOperation::Publish => {
                    self.set_publish_feedback(
                        connection_id,
                        WorkflowFeedback::error(message.clone()),
                    );
                }
                MqttOperation::Subscribe | MqttOperation::Unsubscribe => {
                    self.set_subscribe_feedback(
                        connection_id,
                        WorkflowFeedback::error(message.clone()),
                    );
                }
                _ => {}
            }
            self.update_connection_state(
                connection_id,
                ConnectionState::Error,
                None,
                format!("{} failed", failure.operation.label()),
            );
        } else {
            match failure.operation {
                MqttOperation::Publish => {
                    self.snapshot.workbench.publish.feedback =
                        Some(WorkflowFeedback::error(message.clone()));
                    self.mark_active_workbench_dirty();
                }
                MqttOperation::Subscribe | MqttOperation::Unsubscribe => {
                    self.snapshot.workbench.subscribe.feedback =
                        Some(WorkflowFeedback::error(message.clone()));
                    self.mark_active_workbench_dirty();
                }
                _ => {}
            }
        }
        self.push_diagnostic(Diagnostic::error(message));
    }

    fn add_subscription(&mut self, connection_id: correo_mqtt::ConnectionId, row: SubscriptionRow) {
        let workbench = self.workbench_for_connection_mut(connection_id);
        push_recent_unique(&mut workbench.subscribe.topic_history, &row.topic_filter);
        if let Some(existing) = workbench
            .subscribe
            .subscriptions
            .iter_mut()
            .find(|subscription| subscription.topic_filter == row.topic_filter)
        {
            existing.qos = row.qos;
            existing.active = true;
            existing.messages_visible = true;
            self.mark_workbench_dirty(connection_id);
            return;
        }
        workbench.subscribe.subscriptions.insert(0, row);
        self.mark_workbench_dirty(connection_id);
    }

    fn remove_subscription(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        topic_filter: &str,
    ) {
        self.workbench_for_connection_mut(connection_id)
            .subscribe
            .subscriptions
            .retain(|subscription| subscription.topic_filter != topic_filter);
        self.mark_workbench_dirty(connection_id);
    }

    fn add_publish_success(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        topic: &str,
        payload: Vec<u8>,
        qos: QosLevel,
        retained: bool,
        diagnostics: Vec<MessageDiagnosticRow>,
    ) {
        let workbench = self.workbench_for_connection_mut(connection_id);
        let byte_size = payload.len();
        push_recent_unique(&mut workbench.publish.topic_history, topic);
        let id = next_publish_history_id(workbench);
        let mut badges = Vec::new();
        if retained {
            badges.push("retained".to_owned());
        }
        workbench.publish.history.insert(
            0,
            PublishHistoryRow {
                id,
                topic: topic.to_owned(),
                timestamp: current_timestamp(),
                qos,
                retained,
                payload_preview: payload_preview(&payload),
                payload,
                byte_size,
                badges,
                diagnostics,
            },
        );
        workbench.publish.history.truncate(MAX_PUBLISH_HISTORY_ROWS);
        workbench.publish.selected_history_id = Some(id);
        workbench.publish.feedback = Some(WorkflowFeedback::info(format!(
            "Published {byte_size} bytes to {topic}."
        )));
        self.mark_workbench_dirty(connection_id);
    }

    fn set_publish_feedback(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        feedback: WorkflowFeedback,
    ) {
        self.workbench_for_connection_mut(connection_id)
            .publish
            .feedback = Some(feedback);
        self.mark_workbench_dirty(connection_id);
    }

    fn set_subscribe_feedback(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        feedback: WorkflowFeedback,
    ) {
        self.workbench_for_connection_mut(connection_id)
            .subscribe
            .feedback = Some(feedback);
        self.mark_workbench_dirty(connection_id);
    }

    pub(super) fn refresh_publish_validation(&mut self) {
        let publish = &mut self.snapshot.workbench.publish;
        let topic = publish.topic.trim();
        publish.validation = publish_validation(topic, publish.payload.len());
        publish.valid = TopicName::new(topic).is_ok();
    }

    fn refresh_subscribe_validation(&mut self) {
        let subscribe = &mut self.snapshot.workbench.subscribe;
        let topic = subscribe.topic.trim();
        subscribe.validation = subscribe_validation(topic);
        subscribe.valid = TopicFilter::new(topic).is_ok();
    }
}

fn connection_label(model: &AppModel, connection_id: correo_mqtt::ConnectionId) -> String {
    model
        .snapshot
        .connections
        .iter()
        .find(|connection| connection.id == connection_id)
        .map(|connection| connection.name.clone())
        .unwrap_or_else(|| "unknown connection".to_owned())
}

fn qos_level(qos: Qos) -> QosLevel {
    match qos {
        Qos::AtMostOnce => QosLevel::Zero,
        Qos::AtLeastOnce => QosLevel::One,
        Qos::ExactlyOnce => QosLevel::Two,
    }
}

fn payload_preview(payload: &[u8]) -> String {
    const LIMIT: usize = 96;
    let mut preview = String::from_utf8_lossy(payload).replace(['\n', '\r'], " ");
    if preview.len() > LIMIT {
        let truncated = preview.chars().take(LIMIT).collect::<String>();
        preview = format!("{truncated}...");
    }
    preview
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| OffsetDateTime::now_utc().unix_timestamp().to_string())
}

fn incoming_badges(message: &IncomingMessage) -> Vec<String> {
    let mut badges = Vec::new();
    if message.retain {
        badges.push("retained".to_owned());
    }
    if message.duplicate {
        badges.push("duplicate".to_owned());
    }
    badges
}

fn publish_validation(topic: &str, payload_len: usize) -> Vec<String> {
    let topic_message = match TopicName::new(topic) {
        Ok(_) => "Topic is valid".to_owned(),
        Err(error) => format!("Topic error: {}", error.to_report().message),
    };
    vec![topic_message, format!("Payload: {payload_len} bytes")]
}

fn subscribe_validation(topic: &str) -> Vec<String> {
    vec![match TopicFilter::new(topic) {
        Ok(_) => "Topic filter is valid".to_owned(),
        Err(error) => format!("Topic filter error: {}", error.to_report().message),
    }]
}

fn next_message_id(workbench: &WorkbenchSnapshot) -> u32 {
    workbench
        .messages
        .iter()
        .map(|message| message.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn next_publish_history_id(workbench: &WorkbenchSnapshot) -> u32 {
    workbench
        .publish
        .history
        .iter()
        .map(|row| row.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn push_recent_unique(entries: &mut Vec<String>, topic: &str) {
    entries.retain(|entry| entry != topic);
    entries.insert(0, topic.to_owned());
    entries.truncate(100);
}

fn topic_matches_filter(topic: &str, filter: &str) -> bool {
    let topic_levels = topic.split('/').collect::<Vec<_>>();
    let filter_levels = filter.split('/').collect::<Vec<_>>();

    for (index, filter_level) in filter_levels.iter().enumerate() {
        match *filter_level {
            "#" => return index == filter_levels.len() - 1,
            "+" => {
                if topic_levels.get(index).is_none() {
                    return false;
                }
            }
            literal if topic_levels.get(index) != Some(&literal) => return false,
            _ => {}
        }
    }

    topic_levels.len() == filter_levels.len()
}

fn decrement_matching_subscriptions(workbench: &mut WorkbenchSnapshot, topic: &str) {
    for subscription in &mut workbench.subscribe.subscriptions {
        if subscription.active
            && subscription.message_count > 0
            && topic_matches_filter(topic, &subscription.topic_filter)
        {
            subscription.message_count -= 1;
        }
    }
}
