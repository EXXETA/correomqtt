use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use correo_storage::current::{HistoryStore, Message};
use thiserror::Error;

use crate::WorkbenchSnapshot;

const WORKBENCH_COALESCE_WINDOW: Duration = Duration::from_millis(20);

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

enum QueuedHistoryCommand {
    Incoming(HistoryPersistenceCommand),
    Coalesced(HistoryPersistenceCommand),
}

impl HistoryPersistenceWorker {
    pub fn start(root: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let store = HistoryStore::new(root.into());

        std::thread::spawn(move || {
            let mut deferred = VecDeque::new();
            while let Some(command) = deferred
                .pop_front()
                .or_else(|| receiver.recv().ok().map(QueuedHistoryCommand::Incoming))
            {
                match command {
                    QueuedHistoryCommand::Incoming(
                        command @ HistoryPersistenceCommand::ReplaceWorkbench { .. },
                    ) => coalesce_workbench_replacements(command, &receiver, &mut deferred),
                    QueuedHistoryCommand::Incoming(command)
                    | QueuedHistoryCommand::Coalesced(command) => {
                        let event = apply_history_command(&store, command);
                        let _ = events_sender.send(event);
                    }
                }
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

fn coalesce_workbench_replacements(
    command: HistoryPersistenceCommand,
    receiver: &Receiver<HistoryPersistenceCommand>,
    deferred: &mut VecDeque<QueuedHistoryCommand>,
) {
    let HistoryPersistenceCommand::ReplaceWorkbench {
        connection_id,
        workbench,
    } = command
    else {
        return;
    };

    let deadline = Instant::now() + WORKBENCH_COALESCE_WINDOW;
    let mut replacements = HashMap::from([(connection_id.clone(), workbench)]);
    let mut replacement_order = vec![connection_id];

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }

        let Ok(next) = receiver.recv_timeout(remaining) else {
            break;
        };

        match next {
            HistoryPersistenceCommand::ReplaceWorkbench {
                connection_id,
                workbench,
            } => {
                if let Some(replacement) = replacements.get_mut(&connection_id) {
                    *replacement = workbench;
                } else {
                    replacement_order.push(connection_id.clone());
                    replacements.insert(connection_id, workbench);
                }
            }
            other => deferred.push_back(QueuedHistoryCommand::Incoming(other)),
        }
    }

    for connection_id in replacement_order {
        deferred.push_back(QueuedHistoryCommand::Coalesced(
            HistoryPersistenceCommand::ReplaceWorkbench {
                workbench: replacements
                    .remove(&connection_id)
                    .expect("replacement order only contains queued connections"),
                connection_id,
            },
        ));
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

