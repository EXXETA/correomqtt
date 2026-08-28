use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::stream::BoxStream;
use futures::{stream, StreamExt};
use tokio::sync::{broadcast, oneshot};

use crate::{
    IncomingMessage, MqttConnectionOptions, MqttError, MqttResult, MqttSessionEvent, SessionState,
};

pub(crate) type SharedState = Arc<Mutex<SessionState>>;
pub(crate) type StartupSignal = oneshot::Sender<MqttResult<()>>;

#[derive(Clone)]
pub(crate) struct SessionChannels {
    state: SharedState,
    events: broadcast::Sender<MqttSessionEvent>,
    incoming: broadcast::Sender<Result<IncomingMessage, MqttError>>,
}

impl SessionChannels {
    pub(crate) fn new() -> Self {
        let (events, _) = broadcast::channel(256);
        let (incoming, _) = broadcast::channel(256);
        Self {
            state: Arc::new(Mutex::new(SessionState::Disconnected)),
            events,
            incoming,
        }
    }

    pub(crate) fn current_state(&self) -> SessionState {
        self.state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_else(|_| SessionState::Faulted {
                error: MqttError::io("MQTT session state lock was poisoned").to_report(),
            })
    }

    pub(crate) fn event_stream(&self) -> BoxStream<'static, MqttSessionEvent> {
        broadcast_stream(self.events.subscribe(), |dropped| {
            MqttSessionEvent::Error(broadcast_lag_error(dropped).to_report())
        })
    }

    pub(crate) fn incoming_stream(&self) -> BoxStream<'static, Result<IncomingMessage, MqttError>> {
        broadcast_stream(self.incoming.subscribe(), |dropped| {
            Err(broadcast_lag_error(dropped))
        })
    }

    pub(crate) fn set_state(&self, state: SessionState) {
        if let Ok(mut current) = self.state.lock() {
            *current = state.clone();
        }
        let _ = self.events.send(MqttSessionEvent::StateChanged(state));
    }

    pub(crate) fn report_error(&self, error: MqttError) {
        let _ = self.events.send(MqttSessionEvent::Error(error.to_report()));
    }

    pub(crate) fn report_incoming(&self, message: IncomingMessage) {
        let _ = self.incoming.send(Ok(message.clone()));
        let _ = self.events.send(MqttSessionEvent::Incoming(message));
    }

    pub(crate) fn report_incoming_error(&self, error: MqttError) {
        let _ = self.incoming.send(Err(error.clone()));
        self.report_error(error);
    }

    pub(crate) fn report_published(&self, event: MqttSessionEvent) {
        let _ = self.events.send(event);
    }
}

pub(crate) fn finish_startup(startup: &mut Option<StartupSignal>, result: MqttResult<()>) {
    if let Some(sender) = startup.take() {
        let _ = sender.send(result);
    }
}

pub(crate) fn client_id(options: &MqttConnectionOptions) -> MqttResult<String> {
    match (&options.client_id, options.clean_start) {
        (Some(client_id), _) => Ok(client_id.clone()),
        (None, true) => Ok(String::new()),
        (None, false) => Err(MqttError::invalid_options(
            "persistent MQTT sessions require an explicit client id",
        )),
    }
}

pub(crate) fn keep_alive_seconds(duration: Duration) -> MqttResult<u16> {
    let mut seconds = duration.as_secs();
    if duration.subsec_nanos() > 0 {
        seconds = seconds.saturating_add(1);
    }

    u16::try_from(seconds).map_err(|_| {
        MqttError::invalid_options("MQTT keep alive must fit in a 16-bit seconds value")
    })
}

fn broadcast_lag_error(dropped: u64) -> MqttError {
    MqttError::protocol(format!(
        "MQTT broadcast stream lagged; {dropped} messages dropped"
    ))
}

fn broadcast_stream<T, F>(receiver: broadcast::Receiver<T>, lagged: F) -> BoxStream<'static, T>
where
    T: Clone + Send + 'static,
    F: Fn(u64) -> T + Send + 'static,
{
    stream::unfold((receiver, lagged), |(mut receiver, lagged)| async move {
        match receiver.recv().await {
            Ok(item) => Some((item, (receiver, lagged))),
            Err(broadcast::error::RecvError::Lagged(dropped)) => {
                Some((lagged(dropped), (receiver, lagged)))
            }
            Err(broadcast::error::RecvError::Closed) => None,
        }
    })
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::SessionChannels;
    use crate::{
        ConnectionId, IncomingMessage, MqttErrorKind, MqttSessionEvent, Qos, SessionState,
        TopicName,
    };
    use futures::StreamExt;

    #[tokio::test]
    async fn event_stream_reports_broadcast_lag_with_dropped_count() {
        let channels = SessionChannels::new();
        let mut events = channels.event_stream();

        for _ in 0..257 {
            channels.set_state(SessionState::Connecting);
        }

        let Some(MqttSessionEvent::Error(report)) = events.next().await else {
            panic!("expected lag error event");
        };
        assert_eq!(report.kind, MqttErrorKind::Protocol);
        assert!(
            report
                .message
                .contains("MQTT broadcast stream lagged; 1 messages dropped"),
            "{}",
            report.message
        );
        assert!(matches!(
            events.next().await,
            Some(MqttSessionEvent::StateChanged(SessionState::Connecting))
        ));
    }

    #[tokio::test]
    async fn incoming_stream_reports_broadcast_lag_with_dropped_count() {
        let channels = SessionChannels::new();
        let mut incoming = channels.incoming_stream();
        let message = IncomingMessage {
            connection_id: ConnectionId::new(),
            topic: TopicName::new("lag/test").unwrap(),
            payload: Vec::new(),
            qos: Qos::AtMostOnce,
            retain: false,
            duplicate: false,
            packet_id: None,
        };

        for _ in 0..257 {
            channels.report_incoming(message.clone());
        }

        let Some(Err(error)) = incoming.next().await else {
            panic!("expected lag error");
        };
        assert_eq!(error.kind(), MqttErrorKind::Protocol);
        let diagnostic_message = error.diagnostic_message();
        assert!(
            diagnostic_message.contains("MQTT broadcast stream lagged; 1 messages dropped"),
            "{diagnostic_message}"
        );
        let Some(Ok(delivered)) = incoming.next().await else {
            panic!("expected preserved incoming message");
        };
        assert_eq!(delivered, message);
    }
}
