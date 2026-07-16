#[test]
fn session_event_lag_report_is_visible_in_core_diagnostics() {
    let mut runtime = AppRuntime::new();
    let connection_id = runtime.snapshot().connections[1].id;
    let event = MqttEvent::from_session_event(
        connection_id,
        MqttSessionEvent::Error(
            MqttError::protocol("MQTT broadcast stream lagged; 7 messages dropped").to_report(),
        ),
    );

    runtime
        .event_sender()
        .emit(AppEvent::Mqtt(event))
        .expect("runtime event receiver is available");
    runtime.pump();

    let diagnostic = runtime
        .snapshot()
        .diagnostics
        .first()
        .expect("lag diagnostic is visible");
    assert!(diagnostic
        .message
        .contains("MQTT broadcast stream lagged; 7 messages dropped"));
    assert_eq!(
        connection_state(&runtime, connection_id),
        ConnectionState::Error
    );
}

async fn next_mqtt_event(
    service: &MqttService,
    predicate: impl Fn(&MqttEvent) -> bool,
) -> MqttEvent {
    loop {
        match service.try_recv_event() {
            Ok(event) if predicate(&event) => return event,
            Ok(_) | Err(flume::TryRecvError::Empty) => tokio::task::yield_now().await,
            Err(flume::TryRecvError::Disconnected) => panic!("MQTT event stream disconnected"),
        }
    }
}

#[tokio::test]
async fn lifecycle_operations_time_out_for_hanging_sessions() {
    let connection_id = correo_mqtt::ConnectionId::new();
    let operation_timeout = Duration::from_millis(10);

    let connect_service = MqttService::spawn_with_operation_timeout(
        FakeFactory::new(Arc::default(), None).with_hanging_connect(),
        operation_timeout,
    )
    .unwrap();
    connect_service
        .command_sender()
        .send(MqttCommand::Connect {
            options: connection_options(connection_id),
        })
        .unwrap();
    let connect_event = tokio::time::timeout(
        Duration::from_millis(250),
        next_mqtt_event(&connect_service, |event| {
            matches!(
                event,
                MqttEvent::Failure(failure)
                    if failure.operation == crate::MqttOperation::Connect
            )
        }),
    )
    .await
    .expect("connect timeout produces a failure event");
    assert!(matches!(connect_event, MqttEvent::Failure(_)));

    let disconnect_service = MqttService::spawn_with_operation_timeout(
        FakeFactory::new(Arc::default(), None).with_hanging_disconnect(),
        operation_timeout,
    )
    .unwrap();
    disconnect_service
        .command_sender()
        .send(MqttCommand::Connect {
            options: connection_options(connection_id),
        })
        .unwrap();
    tokio::time::timeout(
        Duration::from_millis(250),
        next_mqtt_event(&disconnect_service, |event| {
            matches!(event, MqttEvent::Connected { .. })
        }),
    )
    .await
    .expect("test session connects");
    disconnect_service
        .command_sender()
        .send(MqttCommand::Disconnect { connection_id })
        .unwrap();
    let disconnect_event = tokio::time::timeout(
        Duration::from_millis(250),
        next_mqtt_event(&disconnect_service, |event| {
            matches!(
                event,
                MqttEvent::Failure(failure)
                    if failure.operation == crate::MqttOperation::Disconnect
            )
        }),
    )
    .await
    .expect("disconnect timeout produces a failure event");
    assert!(matches!(disconnect_event, MqttEvent::Failure(_)));

    let reconnect_service = MqttService::spawn_with_operation_timeout(
        FakeFactory::new(Arc::default(), None).with_hanging_disconnect(),
        operation_timeout,
    )
    .unwrap();
    reconnect_service
        .command_sender()
        .send(MqttCommand::Connect {
            options: connection_options(connection_id),
        })
        .unwrap();
    tokio::time::timeout(
        Duration::from_millis(250),
        next_mqtt_event(&reconnect_service, |event| {
            matches!(event, MqttEvent::Connected { .. })
        }),
    )
    .await
    .expect("test session connects");
    reconnect_service
        .command_sender()
        .send(MqttCommand::Reconnect {
            options: connection_options(connection_id),
        })
        .unwrap();
    let reconnect_event = tokio::time::timeout(
        Duration::from_millis(250),
        next_mqtt_event(&reconnect_service, |event| {
            matches!(
                event,
                MqttEvent::Failure(failure)
                    if failure.operation == crate::MqttOperation::Reconnect
            )
        }),
    )
    .await
    .expect("replacement close timeout produces a reconnect failure event");
    assert!(matches!(reconnect_event, MqttEvent::Failure(_)));
}

#[tokio::test]
async fn shutdown_cancels_a_hanging_session_after_its_budget() {
    let connection_id = correo_mqtt::ConnectionId::new();
    let service = MqttService::spawn_with_operation_timeout(
        FakeFactory::new(Arc::default(), None).with_hanging_disconnect(),
        Duration::from_millis(10),
    )
    .unwrap();
    service
        .command_sender()
        .send(MqttCommand::Connect {
            options: connection_options(connection_id),
        })
        .unwrap();
    tokio::time::timeout(
        Duration::from_millis(250),
        next_mqtt_event(&service, |event| {
            matches!(event, MqttEvent::Connected { .. })
        }),
    )
    .await
    .expect("test session connects");

    assert!(
        tokio::time::timeout(Duration::from_millis(250), service.shutdown())
            .await
            .is_ok(),
        "service shutdown must remain bounded when disconnect hangs"
    );
}
