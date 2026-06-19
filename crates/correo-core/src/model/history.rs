use correo_mqtt::{ConnectionId, Qos};
use correo_storage::current::{Message, MessageType, PublishStatus};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{
    AppCommand, AppModel, HistoryPersistenceCommand, MqttEvent, PublishHistoryRow, QosLevel,
};

impl AppModel {
    pub(crate) fn history_commands_for_mqtt_event(
        &self,
        event: &MqttEvent,
    ) -> Vec<HistoryPersistenceCommand> {
        match event {
            MqttEvent::Published {
                connection_id,
                topic,
                payload,
                qos,
                retain,
                ..
            } => vec![HistoryPersistenceCommand::RecordPublish {
                connection_id: self.storage_connection_id(*connection_id),
                message: Message {
                    topic: topic.as_str().to_owned(),
                    payload: Some(String::from_utf8_lossy(payload).into_owned()),
                    retained: *retain,
                    qos: Some(storage_qos(*qos)),
                    date_time: Some(current_timestamp()),
                    message_id: None,
                    message_type: Some(MessageType::Outgoing),
                    publish_status: Some(PublishStatus::Succeeded),
                },
            }],
            MqttEvent::Subscribed {
                connection_id,
                subscription,
            } => vec![HistoryPersistenceCommand::RecordSubscription {
                connection_id: self.storage_connection_id(*connection_id),
                topic: subscription.topic_filter.as_str().to_owned(),
                hidden: false,
            }],
            _ => Vec::new(),
        }
    }

    pub(crate) fn history_commands_for_app_command(
        &self,
        command: &AppCommand,
    ) -> Vec<HistoryPersistenceCommand> {
        let Some(connection_id) = self.snapshot.selected_connection else {
            return Vec::new();
        };
        let storage_connection_id = self.storage_connection_id(connection_id);
        match command {
            AppCommand::RemovePublishHistoryMessage(message_id) => self
                .snapshot
                .workbench
                .publish
                .history
                .iter()
                .find(|row| row.id == *message_id)
                .map(|row| {
                    vec![HistoryPersistenceCommand::RemovePublishedMessage {
                        connection_id: storage_connection_id,
                        message: published_message_from_row(row),
                    }]
                })
                .unwrap_or_default(),
            AppCommand::ClearPublishHistory => {
                vec![HistoryPersistenceCommand::ClearPublishedMessages {
                    connection_id: storage_connection_id,
                }]
            }
            _ => Vec::new(),
        }
    }

    pub(crate) fn storage_connection_id(&self, connection_id: ConnectionId) -> String {
        self.storage_connection_ids
            .get(&connection_id)
            .cloned()
            .unwrap_or_else(|| connection_id.to_string())
    }
}

fn published_message_from_row(row: &PublishHistoryRow) -> Message {
    Message {
        topic: row.topic.clone(),
        payload: Some(String::from_utf8_lossy(&row.payload).into_owned()),
        retained: row.retained,
        qos: Some(storage_qos_level(row.qos)),
        date_time: Some(row.timestamp.clone()),
        message_id: None,
        message_type: Some(MessageType::Outgoing),
        publish_status: Some(PublishStatus::Succeeded),
    }
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| OffsetDateTime::now_utc().unix_timestamp().to_string())
}

fn storage_qos(qos: Qos) -> correo_storage::current::Qos {
    match qos {
        Qos::AtMostOnce => correo_storage::current::Qos::AtMostOnce,
        Qos::AtLeastOnce => correo_storage::current::Qos::AtLeastOnce,
        Qos::ExactlyOnce => correo_storage::current::Qos::ExactlyOnce,
    }
}

fn storage_qos_level(qos: QosLevel) -> correo_storage::current::Qos {
    match qos {
        QosLevel::Zero => correo_storage::current::Qos::AtMostOnce,
        QosLevel::One => correo_storage::current::Qos::AtLeastOnce,
        QosLevel::Two => correo_storage::current::Qos::ExactlyOnce,
    }
}
