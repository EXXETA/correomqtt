use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use correo_mqtt::{MqttConnectionOptions, MqttError, MqttSession, RumqttSession};
use flume::{Receiver, Sender};
use futures::StreamExt;
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use super::{MqttCommand, MqttEvent, MqttFailure, MqttOperation};
use crate::MessageDiagnosticRow;

const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);
const MQTT_COMMAND_CAPACITY: usize = 256;
const MQTT_EVENT_CAPACITY: usize = 1024;
type PendingPublishDiagnostics =
    Arc<Mutex<HashMap<correo_mqtt::ConnectionId, VecDeque<Vec<MessageDiagnosticRow>>>>>;

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
    shutdown_timeout: Duration,
}

impl MqttService {
    pub fn spawn(factory: impl MqttSessionFactory) -> Result<Self, MqttServiceError> {
        Self::spawn_with_timeouts(factory, DEFAULT_OPERATION_TIMEOUT, DEFAULT_SHUTDOWN_TIMEOUT)
    }

    #[cfg(test)]
    pub(crate) fn spawn_with_operation_timeout(
        factory: impl MqttSessionFactory,
        operation_timeout: Duration,
    ) -> Result<Self, MqttServiceError> {
        Self::spawn_with_timeouts(factory, operation_timeout, operation_timeout)
    }

    fn spawn_with_timeouts(
        factory: impl MqttSessionFactory,
        operation_timeout: Duration,
        shutdown_timeout: Duration,
    ) -> Result<Self, MqttServiceError> {
        tokio::runtime::Handle::try_current().map_err(|_| MqttServiceError::MissingRuntime)?;

        let (command_sender, command_receiver) = flume::bounded(MQTT_COMMAND_CAPACITY);
        let (event_sender, event_receiver) = flume::bounded(MQTT_EVENT_CAPACITY);
        let service_loop = ServiceLoop {
            factory: Arc::new(factory),
            commands: command_receiver,
            events: event_sender,
            sessions: HashMap::new(),
            operation_timeout,
            pending_publish_diagnostics: Arc::new(Mutex::new(HashMap::new())),
        };
        let task = tokio::spawn(service_loop.run());

        Ok(Self {
            commands: MqttCommandSender::new(command_sender),
            events: event_receiver,
            task,
            shutdown_timeout,
        })
    }

    pub fn command_sender(&self) -> MqttCommandSender {
        self.commands.clone()
    }

    pub async fn shutdown(mut self) {
        let _ = self.commands.send(MqttCommand::Shutdown);
        if timeout(self.shutdown_timeout, &mut self.task)
            .await
            .is_err()
        {
            self.task.abort();
            let _ = (&mut self.task).await;
        }
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
        self.sender.try_send(command).map_err(|error| match error {
            flume::TrySendError::Full(command) => MqttServiceSendError::Full(Box::new(command)),
            flume::TrySendError::Disconnected(command) => {
                MqttServiceSendError::Disconnected(Box::new(command))
            }
        })
    }
}

#[derive(Debug, Error)]
pub enum MqttServiceError {
    #[error("MQTT service requires a Tokio runtime")]
    MissingRuntime,
}

#[derive(Debug, Error)]
pub enum MqttServiceSendError {
    #[error("MQTT service command queue is full")]
    Full(Box<MqttCommand>),
    #[error("MQTT service command receiver is disconnected")]
    Disconnected(Box<MqttCommand>),
}

struct ServiceLoop {
    factory: Arc<dyn MqttSessionFactory>,
    commands: Receiver<MqttCommand>,
    events: Sender<MqttEvent>,
    sessions: HashMap<correo_mqtt::ConnectionId, SessionEntry>,
    operation_timeout: Duration,
    pending_publish_diagnostics: PendingPublishDiagnostics,
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
                diagnostics,
            } => self.publish(connection_id, request, diagnostics).await,
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
        if let Err(error) = self.close_existing(connection_id).await {
            self.fail(Some(connection_id), operation, error);
            return;
        }
        self.connect_new(options, operation).await;
    }

    async fn reconnect(&mut self, options: MqttConnectionOptions, operation: MqttOperation) {
        let connection_id = options.connection_id;
        if let Err(error) = self.close_existing(connection_id).await {
            self.fail(Some(connection_id), operation, error);
            return;
        }
        let _ = self.events.send(MqttEvent::Reconnecting {
            connection_id,
            attempt: 1,
        });
        self.connect_new(options, operation).await;
    }

    async fn connect_new(&mut self, options: MqttConnectionOptions, operation: MqttOperation) {
        let connection_id = options.connection_id;
        let mut session = self.factory.create_session(&options);
        let events = session.events();
        let monitor = spawn_event_monitor(
            connection_id,
            events,
            self.events.clone(),
            Arc::clone(&self.pending_publish_diagnostics),
        );
        self.accept(connection_id, operation);

        match operation_result(
            timeout(self.operation_timeout, session.connect(options)).await,
            operation,
        ) {
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

    async fn disconnect(&mut self, connection_id: correo_mqtt::ConnectionId) {
        self.accept(connection_id, MqttOperation::Disconnect);
        let Some(mut entry) = self.sessions.remove(&connection_id) else {
            let _ = self.events.send(MqttEvent::Disconnected { connection_id });
            return;
        };

        let result = operation_result(
            timeout(self.operation_timeout, entry.session.disconnect()).await,
            MqttOperation::Disconnect,
        );
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
        diagnostics: Vec<MessageDiagnosticRow>,
    ) {
        self.accept(connection_id, MqttOperation::Publish);
        if !self.sessions.contains_key(&connection_id) {
            self.fail(
                Some(connection_id),
                MqttOperation::Publish,
                MqttError::Disconnected,
            );
            return;
        }

        self.push_publish_diagnostics(connection_id, diagnostics);
        let entry = self
            .sessions
            .get_mut(&connection_id)
            .expect("session existence was checked before publishing");
        if let Err(error) = operation_result(
            timeout(self.operation_timeout, entry.session.publish(request)).await,
            MqttOperation::Publish,
        ) {
            self.pop_publish_diagnostics(connection_id);
            self.fail(Some(connection_id), MqttOperation::Publish, error);
        }
    }

    fn push_publish_diagnostics(
        &self,
        connection_id: correo_mqtt::ConnectionId,
        diagnostics: Vec<MessageDiagnosticRow>,
    ) {
        if let Ok(mut pending) = self.pending_publish_diagnostics.lock() {
            pending
                .entry(connection_id)
                .or_default()
                .push_back(diagnostics);
        }
    }

    fn pop_publish_diagnostics(&self, connection_id: correo_mqtt::ConnectionId) {
        if let Ok(mut pending) = self.pending_publish_diagnostics.lock() {
            if let Some(queue) = pending.get_mut(&connection_id) {
                queue.pop_front();
                if queue.is_empty() {
                    pending.remove(&connection_id);
                }
            }
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
            timeout(
                self.operation_timeout,
                entry.session.subscribe(subscription),
            )
            .await,
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

    async fn close_existing(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
    ) -> Result<(), MqttError> {
        let Some(mut entry) = self.sessions.remove(&connection_id) else {
            return Ok(());
        };
        let result = operation_result(
            timeout(self.operation_timeout, entry.session.disconnect()).await,
            MqttOperation::Disconnect,
        );
        entry.monitor.abort();
        result
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

impl Drop for SessionEntry {
    fn drop(&mut self) {
        self.monitor.abort();
    }
}

fn spawn_event_monitor(
    connection_id: correo_mqtt::ConnectionId,
    mut events: futures::stream::BoxStream<'static, correo_mqtt::MqttSessionEvent>,
    sender: Sender<MqttEvent>,
    pending_publish_diagnostics: PendingPublishDiagnostics,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            let mut event = MqttEvent::from_session_event(connection_id, event);
            if let MqttEvent::Published { diagnostics, .. } = &mut event {
                *diagnostics =
                    take_publish_diagnostics(connection_id, &pending_publish_diagnostics);
            }
            let _ = sender.send(event);
        }
    })
}

fn take_publish_diagnostics(
    connection_id: correo_mqtt::ConnectionId,
    pending_publish_diagnostics: &PendingPublishDiagnostics,
) -> Vec<MessageDiagnosticRow> {
    let Ok(mut pending) = pending_publish_diagnostics.lock() else {
        return Vec::new();
    };
    let Some(queue) = pending.get_mut(&connection_id) else {
        return Vec::new();
    };
    let diagnostics = queue.pop_front().unwrap_or_default();
    if queue.is_empty() {
        pending.remove(&connection_id);
    }
    diagnostics
}

#[cfg(test)]
mod session_entry_tests {
    use std::collections::{HashMap, VecDeque};
    use std::future::pending;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use futures::stream::StreamExt;

    use super::{spawn_event_monitor, PendingPublishDiagnostics, SessionEntry};
    use crate::mqtt::test_support::{connection_options, FakeFactory};
    use crate::MqttSessionFactory;

    #[tokio::test]
    async fn dropping_session_entry_aborts_event_monitor() {
        let connection_id = correo_mqtt::ConnectionId::new();
        let (sender, _receiver) = flume::unbounded();
        let pending_diagnostics: PendingPublishDiagnostics =
            Arc::new(Mutex::new(HashMap::<_, VecDeque<_>>::new()));
        let monitor = spawn_event_monitor(
            connection_id,
            futures::stream::once(async { pending().await }).boxed(),
            sender,
            pending_diagnostics,
        );
        let monitor_abort = monitor.abort_handle();
        let session = FakeFactory::new(Arc::default(), None)
            .create_session(&connection_options(connection_id));

        drop(SessionEntry { session, monitor });

        tokio::time::timeout(Duration::from_millis(100), async {
            while !monitor_abort.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("dropping a session entry aborts its monitor");
    }
}
