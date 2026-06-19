use std::time::{Duration, Instant};

use correo_mqtt::{ConnectionId, Qos};
use correo_storage::current::{
    AppConfig, HistoryPersistenceSnapshot, ScriptExecution, ScriptExecutionStatus, ScriptFile,
    ScriptLogLevel, ScriptLogRecord, ScriptPersistenceSnapshot, ScriptStore,
};

use crate::{
    startup_state_from_current, MqttCommand, MqttCommandSender,
    ScriptExecutionStatus as UiScriptExecutionStatus, ScriptingCommand, ScriptingEvent,
    ScriptingWorker, ThemeMode,
};

#[test]
fn worker_runs_reported_promise_sample_and_persists_logs() {
    let temp = tempfile::tempdir().unwrap();
    let worker = ScriptingWorker::start(temp.path());
    let source = "const client = clientFactory.getPromiseClient();\nlogger.info('starting new_script.js');\nqueue.process();\n";

    worker
        .dispatch(ScriptingCommand::Create {
            path: "new_script.js".to_owned(),
            source: source.to_owned(),
        })
        .unwrap();
    assert!(matches!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(ScriptingEvent::Stored { .. })
    ));

    worker
        .dispatch(ScriptingCommand::Run {
            execution_id: "exec-1".to_owned(),
            script_name: "new_script.js".to_owned(),
            script_path: "new_script.js".to_owned(),
            source: source.to_owned(),
            connection_id: Some("connection-1".to_owned()),
        })
        .unwrap();

    let mut saw_log = false;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let event = worker.recv_event_timeout(Duration::from_millis(50));
        match event {
            Some(ScriptingEvent::LogAppended { message, .. }) => {
                saw_log |= message.contains("starting new_script.js");
            }
            Some(ScriptingEvent::ExecutionUpdated { status, error, .. }) => {
                assert_eq!(status, UiScriptExecutionStatus::Succeeded);
                assert_eq!(error, None);
                break;
            }
            Some(_) => {}
            None if Instant::now() > deadline => panic!("script did not finish"),
            None => {}
        }
    }
    assert!(saw_log);

    let snapshot = ScriptStore::new(temp.path()).load_snapshot(200).unwrap();
    assert_eq!(snapshot.files[0].name, "new_script.js");
    assert_eq!(
        snapshot.executions[0].status,
        ScriptExecutionStatus::Succeeded
    );
    assert!(snapshot.logs.iter().any(|log| {
        log.execution_id == "exec-1" && log.message.contains("starting new_script.js")
    }));
    let persisted_log = snapshot
        .logs
        .iter()
        .find(|log| log.execution_id == "exec-1")
        .expect("persisted script log");
    assert!(persisted_log.timestamp.as_deref().is_some_and(|timestamp| {
        timestamp.contains('T')
            && !timestamp
                .chars()
                .all(|character| character.is_ascii_digit())
    }));

    let state = startup_state_from_current(
        AppConfig::default(),
        HistoryPersistenceSnapshot::default(),
        snapshot,
        Vec::new(),
        ThemeMode::Dark,
    );
    assert!(state.snapshot.scripts.log_lines.iter().any(|line| {
        line.execution_id == "exec-1" && line.message.contains("starting new_script.js")
    }));
}

#[test]
fn worker_script_mqtt_bridge_queues_commands_and_runs_callbacks() {
    let temp = tempfile::tempdir().unwrap();
    let connection_id = ConnectionId::new();
    let (commands, received_commands) = flume::unbounded();
    let worker = ScriptingWorker::start_with_mqtt_sender(
        temp.path(),
        Some(MqttCommandSender::new(commands)),
    );
    let source = r#"
        const client = clientFactory.getAsyncClient();
        client.connect(() => logger.info('connect callback'));
        client.publish(
            'script/out',
            1,
            'payload',
            () => logger.info('publish callback'),
            () => logger.error('publish failed')
        );
        client.subscribe(
            'script/#',
            2,
            () => logger.info('subscribe callback'),
            () => logger.error('subscribe failed'),
            (_message) => logger.info('message callback')
        );
        client.unsubscribeAll();
    "#;

    worker
        .dispatch(ScriptingCommand::Run {
            execution_id: "exec-mqtt".to_owned(),
            script_name: "mqtt.js".to_owned(),
            script_path: "mqtt.js".to_owned(),
            source: source.to_owned(),
            connection_id: Some(connection_id.to_string()),
        })
        .unwrap();

    let mut logs = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let event = worker.recv_event_timeout(Duration::from_millis(50));
        match event {
            Some(ScriptingEvent::LogAppended { message, .. }) => logs.push(message),
            Some(ScriptingEvent::ExecutionUpdated { status, error, .. }) => {
                assert_eq!(status, UiScriptExecutionStatus::Succeeded);
                assert_eq!(error, None);
                break;
            }
            Some(_) => {}
            None if Instant::now() > deadline => panic!("script did not finish"),
            None => {}
        }
    }

    assert!(logs.iter().any(|message| message == "connect callback"));
    assert!(logs.iter().any(|message| message == "publish callback"));
    assert!(logs.iter().any(|message| message == "subscribe callback"));

    let commands = received_commands.drain().collect::<Vec<_>>();
    assert!(matches!(
        commands.as_slice(),
        [
            MqttCommand::Publish {
                connection_id: publish_connection_id,
                request,
                ..
            },
            MqttCommand::Subscribe {
                connection_id: subscribe_connection_id,
                subscription,
            }
        ] if *publish_connection_id == connection_id
            && request.topic.as_str() == "script/out"
            && request.payload == b"payload"
            && request.qos == Qos::AtLeastOnce
            && *subscribe_connection_id == connection_id
            && subscription.topic_filter.as_str() == "script/#"
            && subscription.qos == Qos::ExactlyOnce
    ));
}

#[test]
fn startup_hydrates_scripts_from_persistence_snapshot() {
    let scripts = ScriptPersistenceSnapshot {
        files: vec![ScriptFile::new(
            "stored.js".into(),
            "logger.info('stored');".to_owned(),
        )],
        executions: vec![ScriptExecution {
            execution_id: "exec-stored".to_owned(),
            script_name: "stored.js".to_owned(),
            script_path: "stored.js".into(),
            connection_id: None,
            status: ScriptExecutionStatus::Succeeded,
            error: None,
            started_at: Some("stored-time".to_owned()),
            ended_at: None,
            duration_ms: Some(42),
            cancelled: false,
            log_path: None,
        }],
        logs: vec![ScriptLogRecord {
            execution_id: "exec-stored".to_owned(),
            sequence: 0,
            timestamp: Some("stored-time".to_owned()),
            level: ScriptLogLevel::Info,
            message: "stored log".to_owned(),
        }],
    };

    let state = startup_state_from_current(
        AppConfig::default(),
        HistoryPersistenceSnapshot::default(),
        scripts,
        Vec::new(),
        ThemeMode::Dark,
    );

    assert_eq!(state.snapshot.scripts.selected_script, "stored.js");
    assert_eq!(
        state.snapshot.scripts.scripts[0].source,
        "logger.info('stored');"
    );
    assert_eq!(
        state.snapshot.scripts.executions[0].status,
        UiScriptExecutionStatus::Succeeded
    );
    assert_eq!(
        state.snapshot.scripts.log_lines[0].execution_id,
        "exec-stored"
    );
}

#[test]
fn worker_clear_finished_removes_persisted_execution_and_logs() {
    let temp = tempfile::tempdir().unwrap();
    let store = ScriptStore::new(temp.path());
    store
        .replace_all(&ScriptPersistenceSnapshot {
            files: vec![ScriptFile::new(
                "stored.js".into(),
                "logger.info('stored');".to_owned(),
            )],
            executions: vec![ScriptExecution {
                execution_id: "exec-stored".to_owned(),
                script_name: "stored.js".to_owned(),
                script_path: "stored.js".into(),
                connection_id: None,
                status: ScriptExecutionStatus::Succeeded,
                error: None,
                started_at: Some("stored-time".to_owned()),
                ended_at: None,
                duration_ms: Some(42),
                cancelled: false,
                log_path: None,
            }],
            logs: vec![ScriptLogRecord {
                execution_id: "exec-stored".to_owned(),
                sequence: 0,
                timestamp: Some("stored-time".to_owned()),
                level: ScriptLogLevel::Info,
                message: "stored log".to_owned(),
            }],
        })
        .unwrap();

    let worker = ScriptingWorker::start(temp.path());
    worker
        .dispatch(ScriptingCommand::ClearFinished {
            executions: vec![("stored.js".to_owned(), "exec-stored".to_owned())],
        })
        .unwrap();

    assert!(matches!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(ScriptingEvent::Stored { .. })
    ));
    let snapshot = ScriptStore::new(temp.path()).load_snapshot(200).unwrap();
    assert!(snapshot.executions.is_empty());
    assert!(snapshot.logs.is_empty());
}
