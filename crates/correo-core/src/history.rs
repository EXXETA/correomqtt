use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use correo_storage::current::{HistoryStore, Message};
use thiserror::Error;

use crate::WorkbenchSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryPersistenceCommand {
    RecordPublish {
        connection_id: String,
        message: Box<Message>,
    },
    RecordSubscription {
        connection_id: String,
        topic: String,
        hidden: bool,
    },
    RemovePublishedMessage {
        connection_id: String,
        message: Box<Message>,
    },
    ClearPublishedMessages {
        connection_id: String,
    },
    ReplaceWorkbench {
        connection_id: String,
        workbench: Box<WorkbenchSnapshot>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryPersistenceKind {
    Publish,
    Subscription,
    Workbench,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryPersistenceEvent {
    Changed {
        connection_id: String,
        kind: HistoryPersistenceKind,
    },
    Failed {
        connection_id: String,
        kind: HistoryPersistenceKind,
        error: String,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HistoryDispatchError {
    #[error("history persistence worker is stopped")]
    Stopped,
}

#[derive(Debug)]
pub struct HistoryPersistenceWorker {
    sender: Sender<HistoryPersistenceCommand>,
    events: Receiver<HistoryPersistenceEvent>,
}

impl HistoryPersistenceWorker {
    pub fn start(root: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let store = HistoryStore::new(root.into());

        std::thread::spawn(move || {
            while let Ok(command) = receiver.recv() {
                let event = apply_history_command(&store, command);
                let _ = events_sender.send(event);
            }
        });

        Self { sender, events }
    }

    pub fn dispatch(&self, command: HistoryPersistenceCommand) -> Result<(), HistoryDispatchError> {
        self.sender
            .send(command)
            .map_err(|_| HistoryDispatchError::Stopped)
    }

    pub fn try_recv_event(&self) -> Option<HistoryPersistenceEvent> {
        self.events.try_recv().ok()
    }

    pub fn recv_event_timeout(&self, timeout: Duration) -> Option<HistoryPersistenceEvent> {
        self.events.recv_timeout(timeout).ok()
    }
}

fn apply_history_command(
    store: &HistoryStore,
    command: HistoryPersistenceCommand,
) -> HistoryPersistenceEvent {
    let (connection_id, kind, result) = match command {
        HistoryPersistenceCommand::RecordPublish {
            connection_id,
            message,
        } => {
            let result = store
                .record_publish_success(&connection_id, *message)
                .map(|_| ());
            (connection_id, HistoryPersistenceKind::Publish, result)
        }
        HistoryPersistenceCommand::RecordSubscription {
            connection_id,
            topic,
            hidden,
        } => {
            let result = store
                .record_subscription(&connection_id, topic, hidden)
                .map(|_| ());
            (connection_id, HistoryPersistenceKind::Subscription, result)
        }
        HistoryPersistenceCommand::RemovePublishedMessage {
            connection_id,
            message,
        } => {
            let result = store
                .remove_published_message(&connection_id, &message)
                .map(|_| ());
            (connection_id, HistoryPersistenceKind::Publish, result)
        }
        HistoryPersistenceCommand::ClearPublishedMessages { connection_id } => {
            let result = store.clear_published_messages(&connection_id).map(|_| ());
            (connection_id, HistoryPersistenceKind::Publish, result)
        }
        HistoryPersistenceCommand::ReplaceWorkbench {
            connection_id,
            workbench,
        } => {
            let result = store
                .replace_workbench(&connection_id, &workbench)
                .map(|_| ());
            (connection_id, HistoryPersistenceKind::Workbench, result)
        }
    };

    match result {
        Ok(()) => HistoryPersistenceEvent::Changed {
            connection_id,
            kind,
        },
        Err(error) => HistoryPersistenceEvent::Failed {
            connection_id,
            kind,
            error: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use correo_storage::current::{HistoryStore, Message, MessageType, PublishStatus, Qos};

    use super::{HistoryPersistenceCommand, HistoryPersistenceEvent, HistoryPersistenceKind};
    use crate::HistoryPersistenceWorker;

    #[test]
    fn worker_persists_history_commands_off_the_caller_thread() {
        let temp = tempfile::tempdir().unwrap();
        let worker = HistoryPersistenceWorker::start(temp.path());
        let message = Message {
            topic: "alerts/status".to_owned(),
            payload: Some("ok".to_owned()),
            retained: false,
            qos: Some(Qos::AtLeastOnce),
            date_time: Some("2026-06-08T18:15:00".to_owned()),
            message_id: Some("synthetic-message".to_owned()),
            message_type: Some(MessageType::Outgoing),
            publish_status: Some(PublishStatus::Succeeded),
        };

        worker
            .dispatch(HistoryPersistenceCommand::RecordPublish {
                connection_id: "connection-01".to_owned(),
                message: Box::new(message),
            })
            .unwrap();
        worker
            .dispatch(HistoryPersistenceCommand::RecordSubscription {
                connection_id: "connection-01".to_owned(),
                topic: "alerts/#".to_owned(),
                hidden: false,
            })
            .unwrap();
        let mut workbench = crate::WorkbenchSnapshot::default();
        workbench.publish.topic = "alerts/status".to_owned();
        worker
            .dispatch(HistoryPersistenceCommand::ReplaceWorkbench {
                connection_id: "connection-01".to_owned(),
                workbench: Box::new(workbench.clone()),
            })
            .unwrap();

        assert_eq!(
            worker.recv_event_timeout(Duration::from_secs(2)),
            Some(HistoryPersistenceEvent::Changed {
                connection_id: "connection-01".to_owned(),
                kind: HistoryPersistenceKind::Publish,
            })
        );
        assert_eq!(
            worker.recv_event_timeout(Duration::from_secs(2)),
            Some(HistoryPersistenceEvent::Changed {
                connection_id: "connection-01".to_owned(),
                kind: HistoryPersistenceKind::Subscription,
            })
        );
        assert_eq!(
            worker.recv_event_timeout(Duration::from_secs(2)),
            Some(HistoryPersistenceEvent::Changed {
                connection_id: "connection-01".to_owned(),
                kind: HistoryPersistenceKind::Workbench,
            })
        );

        let snapshot = HistoryStore::new(temp.path())
            .load_connection("connection-01")
            .unwrap();
        assert_eq!(snapshot.publish_topics.topics, ["alerts/status"]);
        assert_eq!(snapshot.publish_messages.messages.len(), 1);
        assert_eq!(snapshot.subscriptions.topics, ["alerts/#"]);
        let restored = HistoryStore::new(temp.path())
            .load_workbench::<crate::WorkbenchSnapshot>("connection-01")
            .unwrap();
        assert_eq!(restored.publish.topic, "alerts/status");
    }
}
