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
