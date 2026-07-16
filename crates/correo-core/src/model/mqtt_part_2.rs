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
