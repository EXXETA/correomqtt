use correo_mqtt::{
    ConnectionId, IncomingMessage, MqttConnectionOptions, MqttErrorReport, MqttSessionEvent,
    PublishRequest, Qos, SessionState, Subscription, TopicName, UnsubscribeRequest,
};

use crate::MessageDiagnosticRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MqttCommand {
    Connect {
        options: MqttConnectionOptions,
    },
    Reconnect {
        options: MqttConnectionOptions,
    },
    Disconnect {
        connection_id: ConnectionId,
    },
    Publish {
        connection_id: ConnectionId,
        request: PublishRequest,
        diagnostics: Vec<MessageDiagnosticRow>,
    },
    Subscribe {
        connection_id: ConnectionId,
        subscription: Subscription,
    },
    Unsubscribe {
        connection_id: ConnectionId,
        request: UnsubscribeRequest,
    },
    Shutdown,
}

impl MqttCommand {
    pub fn connection_id(&self) -> Option<ConnectionId> {
        match self {
            Self::Connect { options } | Self::Reconnect { options } => Some(options.connection_id),
            Self::Disconnect { connection_id }
            | Self::Publish { connection_id, .. }
            | Self::Subscribe { connection_id, .. }
            | Self::Unsubscribe { connection_id, .. } => Some(*connection_id),
            Self::Shutdown => None,
        }
    }

    pub fn operation(&self) -> MqttOperation {
        match self {
            Self::Connect { .. } => MqttOperation::Connect,
            Self::Reconnect { .. } => MqttOperation::Reconnect,
            Self::Disconnect { .. } => MqttOperation::Disconnect,
            Self::Publish { .. } => MqttOperation::Publish,
            Self::Subscribe { .. } => MqttOperation::Subscribe,
            Self::Unsubscribe { .. } => MqttOperation::Unsubscribe,
            Self::Shutdown => MqttOperation::Shutdown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MqttEvent {
    CommandAccepted {
        connection_id: ConnectionId,
        operation: MqttOperation,
    },
    Connected {
        connection_id: ConnectionId,
    },
    Disconnected {
        connection_id: ConnectionId,
    },
    Reconnecting {
        connection_id: ConnectionId,
        attempt: u32,
    },
    StateChanged {
        connection_id: ConnectionId,
        state: SessionState,
    },
    IncomingMessage(IncomingMessage),
    Published {
        connection_id: ConnectionId,
        topic: TopicName,
        payload: Vec<u8>,
        qos: Qos,
        retain: bool,
        diagnostics: Vec<MessageDiagnosticRow>,
    },
    Subscribed {
        connection_id: ConnectionId,
        subscription: Subscription,
    },
    Unsubscribed {
        connection_id: ConnectionId,
        request: UnsubscribeRequest,
    },
    Failure(MqttFailure),
    ShutdownComplete,
}

impl MqttEvent {
    pub(crate) fn from_session_event(connection_id: ConnectionId, event: MqttSessionEvent) -> Self {
        match event {
            MqttSessionEvent::StateChanged(SessionState::Reconnecting { attempt }) => {
                Self::Reconnecting {
                    connection_id,
                    attempt,
                }
            }
            MqttSessionEvent::StateChanged(state) => Self::StateChanged {
                connection_id,
                state,
            },
            MqttSessionEvent::Incoming(message) => Self::IncomingMessage(message),
            MqttSessionEvent::Published {
                topic,
                payload,
                qos,
                retain,
            } => Self::Published {
                connection_id,
                topic,
                payload,
                qos,
                retain,
                diagnostics: Vec::new(),
            },
            MqttSessionEvent::Subscribed(subscription) => Self::Subscribed {
                connection_id,
                subscription,
            },
            MqttSessionEvent::Unsubscribed(request) => Self::Unsubscribed {
                connection_id,
                request,
            },
            MqttSessionEvent::Error(report) => Self::Failure(MqttFailure {
                connection_id: Some(connection_id),
                operation: MqttOperation::SessionEvent,
                report,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttFailure {
    pub connection_id: Option<ConnectionId>,
    pub operation: MqttOperation,
    pub report: MqttErrorReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqttOperation {
    Connect,
    Reconnect,
    Disconnect,
    Publish,
    Subscribe,
    Unsubscribe,
    SessionEvent,
    Shutdown,
}

impl MqttOperation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Connect => "connect",
            Self::Reconnect => "reconnect",
            Self::Disconnect => "disconnect",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Unsubscribe => "unsubscribe",
            Self::SessionEvent => "session event",
            Self::Shutdown => "shutdown",
        }
    }
}

impl std::fmt::Display for MqttOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.label())
    }
}
