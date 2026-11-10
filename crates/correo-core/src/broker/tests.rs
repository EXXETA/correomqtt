use super::*;

fn next_broker_event(receiver: &flume::Receiver<AppEvent>) -> BuiltInBrokerEvent {
    match receiver.recv_timeout(std::time::Duration::from_secs(1)) {
        Ok(AppEvent::BuiltInBroker(event)) => event,
        other => panic!("expected a broker event, got {other:?}"),
    }
}

#[test]
fn broker_output_marks_ready_only_after_ready_message() {
    let (sender, receiver) = flume::bounded(2);
    let ready = Arc::new(AtomicBool::new(false));
    let output = format!("starting\n{}\n", builtin_broker_ready_message(1883));

    spawn_broker_output_reader(
        std::io::Cursor::new(output),
        AppEventSender::new(sender),
        1883,
        Arc::clone(&ready),
    );

    assert_eq!(
        next_broker_event(&receiver),
        BuiltInBrokerEvent::Log {
            message: "starting".to_owned()
        }
    );
    assert!(receiver.recv().is_err(), "reader must not emit Started");
    assert!(ready.load(Ordering::Acquire));
}

#[cfg(unix)]
fn worker_with_ready_child(script: &str) -> (BuiltInBrokerWorker, flume::Receiver<AppEvent>) {
    let (sender, receiver) = flume::bounded(8);
    let mut worker = BuiltInBrokerWorker::new(AppEventSender::new(sender));
    worker.child = Some(BuiltInBrokerChild {
        process: Command::new("sh").args(["-c", script]).spawn().unwrap(),
        ready: Arc::new(AtomicBool::new(true)),
        port: 1883,
        started_reported: false,
    });
    (worker, receiver)
}

#[cfg(unix)]
#[test]
fn worker_reports_started_once_then_stopped_and_nothing_after() {
    let (mut worker, receiver) = worker_with_ready_child("sleep 30");

    worker.poll();
    worker.poll();
    worker.stop();
    worker.poll();

    assert_eq!(
        next_broker_event(&receiver),
        BuiltInBrokerEvent::Started { port: 1883 }
    );
    assert!(matches!(
        next_broker_event(&receiver),
        BuiltInBrokerEvent::Stopped { .. }
    ));
    assert!(receiver.try_recv().is_err());
}

#[cfg(unix)]
#[test]
fn worker_reports_failure_instead_of_started_when_ready_child_already_exited() {
    let (mut worker, receiver) = worker_with_ready_child("exit 3");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while worker.child.is_some() && std::time::Instant::now() < deadline {
        worker.poll();
        thread::sleep(std::time::Duration::from_millis(10));
    }

    let events: Vec<AppEvent> = receiver.try_iter().collect();
    assert!(matches!(
        events.as_slice(),
        [AppEvent::BuiltInBroker(BuiltInBrokerEvent::Failed { .. })]
            | [
                AppEvent::BuiltInBroker(BuiltInBrokerEvent::Started { .. }),
                AppEvent::BuiltInBroker(BuiltInBrokerEvent::Failed { .. })
            ]
    ));
}

#[test]
fn broker_snapshot_serialization_never_exposes_password() {
    let snapshot = BuiltInBrokerSnapshot {
        credentials_enabled: true,
        username: "broker".to_owned(),
        password: "snapshot-broker-password".to_owned(),
        ..BuiltInBrokerSnapshot::default()
    };

    let serialized = serde_json::to_string(&snapshot).unwrap();

    assert!(!serialized.contains("snapshot-broker-password"));
    assert!(serde_json::from_str::<serde_json::Value>(&serialized)
        .unwrap()
        .get("password")
        .is_none());

    let mut app_snapshot = crate::AppSnapshot::empty();
    app_snapshot.built_in_broker = snapshot;
    assert!(!serde_json::to_string(&app_snapshot)
        .unwrap()
        .contains("snapshot-broker-password"));
}
