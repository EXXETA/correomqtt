#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{mpsc, Arc};
    use std::time::{Duration, Instant};

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

    #[test]
    fn worker_coalesces_duplicate_workbench_replacements_for_connection() {
        let temp = tempfile::tempdir().unwrap();
        let worker = HistoryPersistenceWorker::start(temp.path());

        for topic in ["first/topic", "second/topic", "latest/topic"] {
            let mut workbench = crate::WorkbenchSnapshot::default();
            workbench.publish.topic = topic.to_owned();
            worker
                .dispatch(HistoryPersistenceCommand::ReplaceWorkbench {
                    connection_id: "connection-01".to_owned(),
                    workbench: Box::new(workbench),
                })
                .unwrap();
        }

        let store = HistoryStore::new(temp.path());
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        loop {
            while let Some(event) = worker.try_recv_event() {
                events.push(event);
            }
            let restored = store
                .load_workbench::<crate::WorkbenchSnapshot>("connection-01")
                .unwrap();
            if restored.publish.topic == "latest/topic" {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "latest workbench replacement was not persisted"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        while let Some(event) = worker.try_recv_event() {
            events.push(event);
        }

        let workbench_events = events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    HistoryPersistenceEvent::Changed {
                        connection_id,
                        kind: HistoryPersistenceKind::Workbench,
                    } if connection_id == "connection-01"
                )
            })
            .count();
        assert_eq!(workbench_events, 1);
    }

    #[test]
    fn worker_persists_history_within_coalescing_bound_during_continuous_replacements() {
        let temp = tempfile::tempdir().unwrap();
        let worker = HistoryPersistenceWorker::start(temp.path());
        let sender = worker.sender.clone();
        let sending = Arc::new(AtomicBool::new(true));
        let replacement_count = Arc::new(AtomicUsize::new(0));

        let mut initial_workbench = crate::WorkbenchSnapshot::default();
        initial_workbench.publish.topic = "initial/topic".to_owned();
        worker
            .dispatch(HistoryPersistenceCommand::ReplaceWorkbench {
                connection_id: "connection-01".to_owned(),
                workbench: Box::new(initial_workbench),
            })
            .unwrap();
        let mut second_initial_workbench = crate::WorkbenchSnapshot::default();
        second_initial_workbench.publish.topic = "initial/other".to_owned();
        worker
            .dispatch(HistoryPersistenceCommand::ReplaceWorkbench {
                connection_id: "connection-02".to_owned(),
                workbench: Box::new(second_initial_workbench),
            })
            .unwrap();

        let (producer_started, producer_ready) = mpsc::sync_channel(0);
        let producer_sending = Arc::clone(&sending);
        let producer_count = Arc::clone(&replacement_count);
        let producer = std::thread::spawn(move || {
            while producer_sending.load(Ordering::Relaxed) {
                let replacement = producer_count.fetch_add(1, Ordering::Relaxed) + 1;
                let mut workbench = crate::WorkbenchSnapshot::default();
                workbench.publish.topic = format!("latest/{replacement}");
                let connection_id = if replacement.is_multiple_of(2) {
                    "connection-02"
                } else {
                    "connection-01"
                };
                sender
                    .send(HistoryPersistenceCommand::ReplaceWorkbench {
                        connection_id: connection_id.to_owned(),
                        workbench: Box::new(workbench),
                    })
                    .unwrap();
                if replacement == 2 {
                    producer_started.send(()).unwrap();
                }
            }
        });

        producer_ready.recv_timeout(Duration::from_secs(2)).unwrap();
        worker
            .dispatch(HistoryPersistenceCommand::RecordPublish {
                connection_id: "connection-01".to_owned(),
                message: Box::new(Message {
                    topic: "alerts/status".to_owned(),
                    payload: Some("ok".to_owned()),
                    retained: false,
                    qos: Some(Qos::AtLeastOnce),
                    date_time: None,
                    message_id: None,
                    message_type: Some(MessageType::Outgoing),
                    publish_status: Some(PublishStatus::Succeeded),
                }),
            })
            .unwrap();
        worker
            .dispatch(HistoryPersistenceCommand::RecordSubscription {
                connection_id: "connection-01".to_owned(),
                topic: "alerts/#".to_owned(),
                hidden: false,
            })
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut persisted_publish = false;
        let mut persisted_subscription = false;
        while Instant::now() < deadline && !(persisted_publish && persisted_subscription) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match worker.recv_event_timeout(remaining) {
                Some(HistoryPersistenceEvent::Changed {
                    kind: HistoryPersistenceKind::Publish,
                    ..
                }) => persisted_publish = true,
                Some(HistoryPersistenceEvent::Changed {
                    kind: HistoryPersistenceKind::Subscription,
                    ..
                }) => persisted_subscription = true,
                Some(_) => {}
                None => break,
            }
        }

        sending.store(false, Ordering::Relaxed);
        producer.join().unwrap();

        assert!(
            persisted_publish,
            "publish persistence exceeded coalescing bound"
        );
        assert!(
            persisted_subscription,
            "subscription persistence exceeded coalescing bound"
        );

        let latest_replacement = replacement_count.load(Ordering::Relaxed);
        let latest_first = if latest_replacement.is_multiple_of(2) {
            latest_replacement - 1
        } else {
            latest_replacement
        };
        let latest_second = latest_replacement - latest_replacement % 2;
        let store = HistoryStore::new(temp.path());
        let workbench_deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let first = store
                .load_workbench::<crate::WorkbenchSnapshot>("connection-01")
                .unwrap();
            let second = store
                .load_workbench::<crate::WorkbenchSnapshot>("connection-02")
                .unwrap();
            if first.publish.topic == format!("latest/{latest_first}")
                && second.publish.topic == format!("latest/{latest_second}")
            {
                break;
            }
            assert!(
                Instant::now() < workbench_deadline,
                "latest workbench replacements were not persisted"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
