use std::sync::{Arc, Mutex};

use correo_mqtt::{MqttError, PublishRequest, Qos, Subscription, UnsubscribeRequest};
use correo_storage::current::HistoryStore;

use super::test_support::{connection_options, connection_state, pump_until, FakeFactory};
use crate::{
    AppCommand, AppRuntime, ConnectionState, HistoryPersistenceWorker, MqttCommand, MqttService,
};

#[tokio::test]
async fn runtime_routes_mqtt_lifecycle_without_blocking_pump() {
    let created_sessions = Arc::new(Mutex::new(0));
    let service = MqttService::spawn(FakeFactory::new(created_sessions.clone(), None)).unwrap();
    let mut runtime = AppRuntime::new();
    runtime.attach_mqtt_service(service);

    let connection_id = runtime.snapshot().connections[1].id;
    let options = connection_options(connection_id);
    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Connect {
            options: options.clone(),
        })))
        .unwrap();

    let first_pump = runtime.pump();
    assert_eq!(first_pump.commands_processed, 1);
    assert_eq!(
        connection_state(&runtime, connection_id),
        ConnectionState::Connecting
    );

    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Reconnect {
            options: options.clone(),
        })))
        .unwrap();
    runtime.pump();
    assert_eq!(
        connection_state(&runtime, connection_id),
        ConnectionState::Reconnecting
    );

    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;
    assert_eq!(*created_sessions.lock().unwrap(), 2);
}

#[tokio::test]
async fn runtime_applies_pub_sub_and_incoming_events() {
    let service = MqttService::spawn(FakeFactory::new(Arc::default(), None)).unwrap();
    let mut runtime = AppRuntime::new();
    runtime.attach_mqtt_service(service);
    let connection_id = runtime.snapshot().connections[1].id;
    runtime
        .command_sender()
        .send(AppCommand::SelectConnection(connection_id))
        .unwrap();
    runtime.pump();

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Connect {
            options: connection_options(connection_id),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Subscribe {
            connection_id,
            subscription: Subscription::new("bridge/#", Qos::AtLeastOnce).unwrap(),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .subscribe
            .subscriptions
            .iter()
            .any(|subscription| subscription.topic_filter == "bridge/#")
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Publish {
            connection_id,
            request: PublishRequest::new(
                "bridge/device-1/state",
                b"online".to_vec(),
                Qos::AtLeastOnce,
                false,
            )
            .unwrap(),
            diagnostics: Vec::new(),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .messages
            .first()
            .is_some_and(|message| message.topic == "bridge/device-1/state")
    })
    .await;

    let subscription = runtime
        .snapshot()
        .workbench
        .subscribe
        .subscriptions
        .iter()
        .find(|subscription| subscription.topic_filter == "bridge/#")
        .unwrap();
    assert_eq!(subscription.message_count, 1);

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Unsubscribe {
            connection_id,
            request: UnsubscribeRequest::new("bridge/#").unwrap(),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .subscribe
            .subscriptions
            .iter()
            .all(|subscription| subscription.topic_filter != "bridge/#")
    })
    .await;
}

#[tokio::test]
async fn runtime_routes_ui_pub_sub_commands_through_mqtt_service() {
    let service = MqttService::spawn(FakeFactory::new(Arc::default(), None)).unwrap();
    let mut runtime = AppRuntime::new();
    runtime.attach_mqtt_service(service);
    let connection_id = runtime.snapshot().connections[2].id;

    runtime
        .command_sender()
        .send(AppCommand::Connect(connection_id))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::UpdateSubscribeTopic("bridge/#".to_owned()))
        .unwrap();
    runtime.pump();
    runtime
        .command_sender()
        .send(AppCommand::Subscribe)
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .subscribe
            .subscriptions
            .iter()
            .any(|subscription| subscription.topic_filter == "bridge/#")
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishTopic(
            "bridge/device-2/state".to_owned(),
        ))
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishPayload("online".to_owned()))
        .unwrap();
    runtime.pump();
    runtime.command_sender().send(AppCommand::Publish).unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .messages
            .first()
            .is_some_and(|message| message.topic == "bridge/device-2/state")
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Unsubscribe("bridge/#".to_owned()))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .workbench
            .subscribe
            .subscriptions
            .iter()
            .all(|subscription| subscription.topic_filter != "bridge/#")
    })
    .await;
}

#[tokio::test]
async fn runtime_persists_successful_ui_publish_and_subscription_history() {
    let temp = tempfile::tempdir().unwrap();
    let service = MqttService::spawn(FakeFactory::new(Arc::default(), None)).unwrap();
    let mut runtime = AppRuntime::new();
    runtime.attach_mqtt_service(service);
    runtime.attach_history_worker(HistoryPersistenceWorker::start(temp.path()));
    let connection_id = runtime.snapshot().connections[2].id;
    let storage_id = connection_id.to_string();

    runtime
        .command_sender()
        .send(AppCommand::Connect(connection_id))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::UpdateSubscribeTopic("persist/#".to_owned()))
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::Subscribe)
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishTopic("persist/device".to_owned()))
        .unwrap();
    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishPayload("stored".to_owned()))
        .unwrap();
    runtime.command_sender().send(AppCommand::Publish).unwrap();

    let store = HistoryStore::new(temp.path());
    pump_until(&mut runtime, |_| {
        store.load_connection(&storage_id).is_ok_and(|history| {
            history
                .subscriptions
                .topics
                .contains(&"persist/#".to_owned())
                && history
                    .publish_messages
                    .messages
                    .iter()
                    .any(|message| message.topic == "persist/device")
        })
    })
    .await;
}

#[tokio::test]
async fn publish_failures_from_ui_commands_are_redacted() {
    let secret = "synthetic-publish-secret";
    let service = MqttService::spawn(
        FakeFactory::new(Arc::default(), None)
            .with_publish_error(MqttError::auth(format!("password={secret}"))),
    )
    .unwrap();
    let mut runtime = AppRuntime::new();
    runtime.attach_mqtt_service(service);
    let connection_id = runtime.snapshot().connections[2].id;

    runtime
        .command_sender()
        .send(AppCommand::Connect(connection_id))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::UpdatePublishTopic("bridge/failure".to_owned()))
        .unwrap();
    runtime.pump();
    runtime.command_sender().send(AppCommand::Publish).unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("MQTT publish failed"))
    })
    .await;

    let diagnostics = format!("{:?}", runtime.snapshot().diagnostics);
    assert!(!diagnostics.contains(secret), "{diagnostics}");
    assert!(diagnostics.contains("[REDACTED]"), "{diagnostics}");
    assert_eq!(
        connection_state(&runtime, connection_id),
        ConnectionState::Error
    );
}

#[tokio::test]
async fn disconnect_cleans_up_service_session() {
    let service = MqttService::spawn(FakeFactory::new(Arc::default(), None)).unwrap();
    let mut runtime = AppRuntime::new();
    runtime.attach_mqtt_service(service);
    let connection_id = runtime.snapshot().connections[2].id;

    runtime
        .command_sender()
        .send(AppCommand::Connect(connection_id))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        connection_state(runtime, connection_id) == ConnectionState::Connected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Disconnect(connection_id))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime.snapshot().active_connection.is_none()
            && connection_state(runtime, connection_id) == ConnectionState::Disconnected
    })
    .await;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Publish {
            connection_id,
            request: PublishRequest::new(
                "bridge/after-disconnect",
                b"offline".to_vec(),
                Qos::AtMostOnce,
                false,
            )
            .unwrap(),
            diagnostics: Vec::new(),
        })))
        .unwrap();
    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("MQTT publish failed"))
    })
    .await;
}

#[tokio::test]
async fn mqtt_failures_are_redacted_before_diagnostics() {
    let secret = "synthetic-secret-value";
    let service = MqttService::spawn(FakeFactory::new(
        Arc::default(),
        Some(MqttError::auth(format!("password={secret}"))),
    ))
    .unwrap();
    let mut runtime = AppRuntime::new();
    runtime.attach_mqtt_service(service);
    let connection_id = runtime.snapshot().connections[1].id;

    runtime
        .command_sender()
        .send(AppCommand::Mqtt(Box::new(MqttCommand::Connect {
            options: connection_options(connection_id),
        })))
        .unwrap();

    pump_until(&mut runtime, |runtime| {
        runtime
            .snapshot()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("MQTT connect failed"))
    })
    .await;

    let diagnostics = format!("{:?}", runtime.snapshot().diagnostics);
    assert!(!diagnostics.contains(secret), "{diagnostics}");
    assert!(diagnostics.contains("[REDACTED]"), "{diagnostics}");
    assert_eq!(
        connection_state(&runtime, connection_id),
        ConnectionState::Error
    );
}
