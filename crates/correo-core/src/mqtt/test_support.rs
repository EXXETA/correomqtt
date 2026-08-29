use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use correo_mqtt::{
    ConnectionId, IncomingMessage, MqttConnectionOptions, MqttEndpoint, MqttError, MqttResult,
    MqttSession, MqttSessionEvent, PublishRequest, SessionState, Subscription, UnsubscribeRequest,
};
use futures::stream::{self, BoxStream};
use futures::StreamExt;

use crate::{AppRuntime, ConnectionState, MqttSessionFactory};

#[derive(Clone)]
pub(crate) struct FakeFactory {
    created_sessions: Arc<Mutex<usize>>,
    connect_error: Option<MqttError>,
    publish_error: Option<MqttError>,
    connect_hangs: bool,
    disconnect_hangs: bool,
}

impl FakeFactory {
    pub(crate) fn new(
        created_sessions: Arc<Mutex<usize>>,
        connect_error: Option<MqttError>,
    ) -> Self {
        Self {
            created_sessions,
            connect_error,
            publish_error: None,
            connect_hangs: false,
            disconnect_hangs: false,
        }
    }

    pub(crate) fn with_publish_error(mut self, error: MqttError) -> Self {
        self.publish_error = Some(error);
        self
    }

    pub(crate) fn with_hanging_connect(mut self) -> Self {
        self.connect_hangs = true;
        self
    }

    pub(crate) fn with_hanging_disconnect(mut self) -> Self {
        self.disconnect_hangs = true;
        self
    }
}

impl MqttSessionFactory for FakeFactory {
    fn create_session(&self, options: &MqttConnectionOptions) -> Box<dyn MqttSession> {
        *self.created_sessions.lock().unwrap() += 1;
        Box::new(FakeSession::new(
            options.connection_id,
            self.connect_error.clone(),
            self.publish_error.clone(),
            self.connect_hangs,
            self.disconnect_hangs,
        ))
    }
}

struct FakeSession {
    connection_id: ConnectionId,
    state: SessionState,
    event_sender: flume::Sender<MqttSessionEvent>,
    event_receiver: Option<flume::Receiver<MqttSessionEvent>>,
    connect_error: Option<MqttError>,
    publish_error: Option<MqttError>,
    connect_hangs: bool,
    disconnect_hangs: bool,
}

impl FakeSession {
    fn new(
        connection_id: ConnectionId,
        connect_error: Option<MqttError>,
        publish_error: Option<MqttError>,
        connect_hangs: bool,
        disconnect_hangs: bool,
    ) -> Self {
        let (event_sender, event_receiver) = flume::unbounded();
        Self {
            connection_id,
            state: SessionState::Disconnected,
            event_sender,
            event_receiver: Some(event_receiver),
            connect_error,
            publish_error,
            connect_hangs,
            disconnect_hangs,
        }
    }

    fn emit(&self, event: MqttSessionEvent) {
        self.event_sender.send(event).unwrap();
    }
}

#[async_trait]
impl MqttSession for FakeSession {
    async fn connect(&mut self, _options: MqttConnectionOptions) -> MqttResult<()> {
        if let Some(error) = &self.connect_error {
            self.emit(MqttSessionEvent::Error(error.to_report()));
            return Err(error.clone());
        }
        if self.connect_hangs {
            std::future::pending::<()>().await;
        }

        self.state = SessionState::Connected;
        self.emit(MqttSessionEvent::StateChanged(SessionState::Connected));
        Ok(())
    }

    async fn disconnect(&mut self) -> MqttResult<()> {
        if self.disconnect_hangs {
            std::future::pending::<()>().await;
        }
        self.state = SessionState::Disconnected;
        self.emit(MqttSessionEvent::StateChanged(SessionState::Disconnected));
        Ok(())
    }

    async fn publish(&mut self, request: PublishRequest) -> MqttResult<()> {
        if let Some(error) = &self.publish_error {
            return Err(error.clone());
        }

        self.emit(MqttSessionEvent::Published {
            topic: request.topic.clone(),
            payload: request.payload.clone(),
            qos: request.qos,
            retain: request.retain,
        });
        self.emit(MqttSessionEvent::Incoming(IncomingMessage {
            connection_id: self.connection_id,
            topic: request.topic,
            payload: request.payload,
            qos: request.qos,
            retain: request.retain,
            duplicate: false,
            packet_id: Some(1),
        }));
        Ok(())
    }

    async fn subscribe(&mut self, subscription: Subscription) -> MqttResult<()> {
        self.emit(MqttSessionEvent::Subscribed(subscription));
        Ok(())
    }

    async fn unsubscribe(&mut self, request: UnsubscribeRequest) -> MqttResult<()> {
        self.emit(MqttSessionEvent::Unsubscribed(request));
        Ok(())
    }

    fn current_state(&self) -> SessionState {
        self.state.clone()
    }

    fn events(&mut self) -> BoxStream<'static, MqttSessionEvent> {
        let Some(receiver) = self.event_receiver.take() else {
            return stream::empty().boxed();
        };
        stream::unfold(receiver, |receiver| async move {
            receiver
                .recv_async()
                .await
                .ok()
                .map(|event| (event, receiver))
        })
        .boxed()
    }

    fn incoming(&mut self) -> BoxStream<'static, Result<IncomingMessage, MqttError>> {
        stream::empty().boxed()
    }
}

pub(crate) fn connection_options(connection_id: ConnectionId) -> MqttConnectionOptions {
    MqttConnectionOptions::new(
        connection_id,
        "fake",
        MqttEndpoint::new("localhost", 1883).unwrap(),
    )
}

pub(crate) fn connection_state(
    runtime: &AppRuntime,
    connection_id: ConnectionId,
) -> ConnectionState {
    runtime
        .snapshot()
        .connections
        .iter()
        .find(|connection| connection.id == connection_id)
        .unwrap()
        .state
}

pub(crate) async fn pump_until(
    runtime: &mut AppRuntime,
    mut condition: impl FnMut(&AppRuntime) -> bool,
) {
    for _ in 0..50 {
        runtime.pump();
        if condition(runtime) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    runtime.pump();
    assert!(condition(runtime), "runtime condition was not reached");
}
