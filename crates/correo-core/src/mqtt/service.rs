use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use correo_mqtt::{MqttConnectionOptions, MqttError, MqttSession, RumqttSession};
use flume::{Receiver, Sender};
use futures::StreamExt;
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use super::{MqttCommand, MqttEvent, MqttFailure, MqttOperation};

const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(10);

pub trait MqttSessionFactory: Send + Sync + 'static {
    fn create_session(&self, options: &MqttConnectionOptions) -> Box<dyn MqttSession>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RumqttSessionFactory;

impl MqttSessionFactory for RumqttSessionFactory {
    fn create_session(&self, options: &MqttConnectionOptions) -> Box<dyn MqttSession> {
        Box::new(RumqttSession::for_protocol(options.protocol_version))
    }
}

#[derive(Debug)]
pub struct MqttService {
    commands: MqttCommandSender,
    events: Receiver<MqttEvent>,
    task: JoinHandle<()>,
}

impl MqttService {
    pub fn spawn(factory: impl MqttSessionFactory) -> Result<Self, MqttServiceError> {
        Self::spawn_with_operation_timeout(factory, DEFAULT_OPERATION_TIMEOUT)
    }

    pub(crate) fn spawn_with_operation_timeout(
        factory: impl MqttSessionFactory,
        operation_timeout: Duration,
    ) -> Result<Self, MqttServiceError> {
        tokio::runtime::Handle::try_current().map_err(|_| MqttServiceError::MissingRuntime)?;

        let (command_sender, command_receiver) = flume::unbounded();
        let (event_sender, event_receiver) = flume::unbounded();
        let service_loop = ServiceLoop {
            factory: Arc::new(factory),
            commands: command_receiver,
            events: event_sender,
            sessions: HashMap::new(),
            operation_timeout,
        };
        let task = tokio::spawn(service_loop.run());

        Ok(Self {
            commands: MqttCommandSender::new(command_sender),
            events: event_receiver,
            task,
        })
    }

    pub fn command_sender(&self) -> MqttCommandSender {
        self.commands.clone()
    }

    pub async fn shutdown(mut self) {
        let _ = self.commands.send(MqttCommand::Shutdown);
        let _ = (&mut self.task).await;
    }

    pub(crate) fn try_recv_event(&self) -> Result<MqttEvent, flume::TryRecvError> {
        self.events.try_recv()
    }
}

impl Drop for MqttService {
    fn drop(&mut self) {
        let _ = self.commands.send(MqttCommand::Shutdown);
        self.task.abort();
    }
}

#[derive(Debug, Clone)]
pub struct MqttCommandSender {
    sender: Sender<MqttCommand>,
}

impl MqttCommandSender {
    pub(crate) fn new(sender: Sender<MqttCommand>) -> Self {
        Self { sender }
    }

    pub fn send(&self, command: MqttCommand) -> Result<(), MqttServiceSendError> {
        self.sender
            .try_send(command)
            .map_err(|error| MqttServiceSendError::Disconnected(Box::new(error.into_inner())))
    }
}

#[derive(Debug, Error)]
pub enum MqttServiceError {
    #[error("MQTT service requires a Tokio runtime")]
    MissingRuntime,
}

#[derive(Debug, Error)]
pub enum MqttServiceSendError {
    #[error("MQTT service command receiver is disconnected")]
    Disconnected(Box<MqttCommand>),
}

struct ServiceLoop {
    factory: Arc<dyn MqttSessionFactory>,
    commands: Receiver<MqttCommand>,
    events: Sender<MqttEvent>,
    sessions: HashMap<correo_mqtt::ConnectionId, SessionEntry>,
    operation_timeout: Duration,
}

impl ServiceLoop {
    async fn run(mut self) {
        while let Ok(command) = self.commands.recv_async().await {
            let shutdown = matches!(command, MqttCommand::Shutdown);
            self.handle_command(command).await;
            if shutdown {
                break;
            }
        }

        self.shutdown_sessions().await;
        let _ = self.events.send(MqttEvent::ShutdownComplete);
    }

    async fn handle_command(&mut self, command: MqttCommand) {
        match command {
            MqttCommand::Connect { options } => self.connect(options, MqttOperation::Connect).await,
            MqttCommand::Reconnect { options } => {
                self.reconnect(options, MqttOperation::Reconnect).await;
            }
            MqttCommand::Disconnect { connection_id } => self.disconnect(connection_id).await,
            MqttCommand::Publish {
                connection_id,
                request,
            } => self.publish(connection_id, request).await,
            MqttCommand::Subscribe {
                connection_id,
                subscription,
            } => self.subscribe(connection_id, subscription).await,
            MqttCommand::Unsubscribe {
                connection_id,
                request,
            } => self.unsubscribe(connection_id, request).await,
            MqttCommand::Shutdown => {}
        }
    }

    async fn connect(&mut self, options: MqttConnectionOptions, operation: MqttOperation) {
        let connection_id = options.connection_id;
        self.close_existing(connection_id).await;

        let mut session = self.factory.create_session(&options);
        let events = session.events();
        let monitor = spawn_event_monitor(connection_id, events, self.events.clone());
        self.accept(connection_id, operation);

        match session.connect(options).await {
            Ok(()) => {
                let _ = self.events.send(MqttEvent::Connected { connection_id });
                self.sessions
                    .insert(connection_id, SessionEntry { session, monitor });
            }
            Err(error) => {
                monitor.abort();
                self.fail(Some(connection_id), operation, error);
            }
        }
    }

    async fn reconnect(&mut self, options: MqttConnectionOptions, operation: MqttOperation) {
        let connection_id = options.connection_id;
        self.close_existing(connection_id).await;
        let _ = self.events.send(MqttEvent::Reconnecting {
            connection_id,
            attempt: 1,
        });
        self.connect(options, operation).await;
    }

    async fn disconnect(&mut self, connection_id: correo_mqtt::ConnectionId) {
        self.accept(connection_id, MqttOperation::Disconnect);
        let Some(mut entry) = self.sessions.remove(&connection_id) else {
            let _ = self.events.send(MqttEvent::Disconnected { connection_id });
            return;
        };

        let result = entry.session.disconnect().await;
        entry.monitor.abort();
        match result {
            Ok(()) => {
                let _ = self.events.send(MqttEvent::Disconnected { connection_id });
            }
            Err(error) => self.fail(Some(connection_id), MqttOperation::Disconnect, error),
        }
    }

    async fn publish(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        request: correo_mqtt::PublishRequest,
    ) {
        self.accept(connection_id, MqttOperation::Publish);
        let Some(entry) = self.sessions.get_mut(&connection_id) else {
            self.fail(
                Some(connection_id),
                MqttOperation::Publish,
                MqttError::Disconnected,
            );
            return;
        };

        if let Err(error) = operation_result(
            timeout(self.operation_timeout, entry.session.publish(request)).await,
            MqttOperation::Publish,
        ) {
            self.fail(Some(connection_id), MqttOperation::Publish, error);
        }
    }

    async fn subscribe(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        subscription: correo_mqtt::Subscription,
    ) {
        self.accept(connection_id, MqttOperation::Subscribe);
        let Some(entry) = self.sessions.get_mut(&connection_id) else {
            self.fail(
                Some(connection_id),
                MqttOperation::Subscribe,
                MqttError::Disconnected,
            );
            return;
        };

        if let Err(error) = operation_result(
            timeout(self.operation_timeout, entry.session.subscribe(subscription)).await,
            MqttOperation::Subscribe,
        ) {
            self.fail(Some(connection_id), MqttOperation::Subscribe, error);
        }
    }

    async fn unsubscribe(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        request: correo_mqtt::UnsubscribeRequest,
    ) {
        self.accept(connection_id, MqttOperation::Unsubscribe);
        let Some(entry) = self.sessions.get_mut(&connection_id) else {
            self.fail(
                Some(connection_id),
                MqttOperation::Unsubscribe,
                MqttError::Disconnected,
            );
            return;
        };

        if let Err(error) = operation_result(
            timeout(self.operation_timeout, entry.session.unsubscribe(request)).await,
            MqttOperation::Unsubscribe,
        ) {
            self.fail(Some(connection_id), MqttOperation::Unsubscribe, error);
        }
    }

    async fn close_existing(&mut self, connection_id: correo_mqtt::ConnectionId) {
        if let Some(mut entry) = self.sessions.remove(&connection_id) {
            let _ = entry.session.disconnect().await;
            entry.monitor.abort();
        }
    }

    async fn shutdown_sessions(&mut self) {
        let connection_ids = self.sessions.keys().copied().collect::<Vec<_>>();
        for connection_id in connection_ids {
            self.disconnect(connection_id).await;
        }
    }

    fn accept(&self, connection_id: correo_mqtt::ConnectionId, operation: MqttOperation) {
        let _ = self.events.send(MqttEvent::CommandAccepted {
            connection_id,
            operation,
        });
    }

    fn fail(
        &self,
        connection_id: Option<correo_mqtt::ConnectionId>,
        operation: MqttOperation,
        error: MqttError,
    ) {
        let _ = self.events.send(MqttEvent::Failure(MqttFailure {
            connection_id,
            operation,
            report: error.to_report(),
        }));
    }
}

fn operation_result(
    result: Result<Result<(), MqttError>, tokio::time::error::Elapsed>,
    operation: MqttOperation,
) -> Result<(), MqttError> {
    match result {
        Ok(result) => result,
        Err(_) => Err(MqttError::protocol(format!(
            "{operation} timed out while waiting for MQTT acknowledgement"
        ))),
    }
}

struct SessionEntry {
    session: Box<dyn MqttSession>,
    monitor: JoinHandle<()>,
}

fn spawn_event_monitor(
    connection_id: correo_mqtt::ConnectionId,
    mut events: futures::stream::BoxStream<'static, correo_mqtt::MqttSessionEvent>,
    sender: Sender<MqttEvent>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            let _ = sender.send(MqttEvent::from_session_event(connection_id, event));
        }
    })
}
