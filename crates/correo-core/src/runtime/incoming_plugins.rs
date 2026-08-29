use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use correo_mqtt::IncomingMessage;
use flume::{Receiver, Sender};

use crate::{
    MessageDiagnosticRow, MqttEvent, PluginDiagnosticSeverity, PluginHookCall, PluginHookExecutor,
    PluginHookInput, PluginHookKind, PluginHookOutput, PluginValidation,
};

use super::plugin_helpers::{
    incoming_from_plugin_message, message_diagnostic, plugin_message_from_incoming,
    plugin_message_from_transport, plugin_transport_message_from_incoming, ActiveHook,
};

const INCOMING_PLUGIN_CAPACITY: usize = 128;

#[derive(Debug)]
pub(super) struct IncomingPluginWorker {
    requests: Sender<IncomingPluginJob>,
    results: Receiver<IncomingPluginResult>,
    cancelled: Arc<AtomicBool>,
    executor: Arc<dyn PluginHookExecutor>,
}

impl IncomingPluginWorker {
    pub(super) fn start(executor: Arc<dyn PluginHookExecutor>) -> Self {
        Self::start_with_capacity(executor, INCOMING_PLUGIN_CAPACITY)
    }

    pub(super) fn start_with_capacity(
        executor: Arc<dyn PluginHookExecutor>,
        capacity: usize,
    ) -> Self {
        let (requests, request_receiver) = flume::bounded(capacity);
        let (result_sender, results) = flume::bounded(capacity);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let worker_executor = Arc::clone(&executor);
        std::thread::Builder::new()
            .name("correo-plugin-ingress".to_owned())
            .spawn(move || {
                while let Ok(job) = request_receiver.recv() {
                    let result = run_job(job, worker_executor.as_ref(), &worker_cancelled);
                    if result_sender.send(result).is_err() {
                        break;
                    }
                }
            })
            .expect("plugin ingress worker thread can start");
        Self {
            requests,
            results,
            cancelled,
            executor,
        }
    }

    pub(super) fn enqueue(
        &self,
        message: IncomingMessage,
        hooks: Vec<ActiveHook>,
        diagnostics: Vec<MessageDiagnosticRow>,
    ) -> Result<(), IncomingPluginQueueError> {
        self.requests
            .try_send(IncomingPluginJob {
                message,
                hooks,
                diagnostics,
            })
            .map_err(|error| match error {
                flume::TrySendError::Full(job) => IncomingPluginQueueError::Full {
                    message: job.message,
                    diagnostics: job.diagnostics,
                },
                flume::TrySendError::Disconnected(job) => IncomingPluginQueueError::Disconnected {
                    message: job.message,
                    diagnostics: job.diagnostics,
                },
            })
    }

    pub(super) fn try_recv(&self) -> Option<IncomingPluginResult> {
        self.results.try_recv().ok()
    }

    pub(super) fn has_pending_results(&self) -> bool {
        !self.results.is_empty()
    }

    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.executor.cancel();
    }
}

impl Drop for IncomingPluginWorker {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[derive(Debug)]
pub(super) enum IncomingPluginQueueError {
    Full {
        message: IncomingMessage,
        diagnostics: Vec<MessageDiagnosticRow>,
    },
    Disconnected {
        message: IncomingMessage,
        diagnostics: Vec<MessageDiagnosticRow>,
    },
}

pub(super) struct IncomingPluginResult {
    pub(super) event: Option<MqttEvent>,
    pub(super) diagnostics: Vec<MessageDiagnosticRow>,
}

struct IncomingPluginJob {
    message: IncomingMessage,
    hooks: Vec<ActiveHook>,
    diagnostics: Vec<MessageDiagnosticRow>,
}

fn run_job(
    job: IncomingPluginJob,
    executor: &dyn PluginHookExecutor,
    cancelled: &AtomicBool,
) -> IncomingPluginResult {
    let mut diagnostics = job.diagnostics;
    if cancelled.load(Ordering::Acquire) {
        diagnostics.push(cancellation_diagnostic());
        return IncomingPluginResult {
            event: None,
            diagnostics,
        };
    }

    let message = plugin_message_from_incoming(&job.message);
    let retained_when_absent = message.retained;
    let mut transport = plugin_transport_message_from_incoming(&job.message, message);
    let transport_capabilities = transport.capabilities.clone();
    for hook in job
        .hooks
        .iter()
        .filter(|hook| hook.hook == PluginHookKind::IncomingTransform)
    {
        if cancelled.load(Ordering::Acquire) {
            diagnostics.push(cancellation_diagnostic());
            return IncomingPluginResult {
                event: None,
                diagnostics,
            };
        }
        let Ok(config) = serde_json::from_str(&hook.config_json) else {
            diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Error,
                "Plugin hook config is invalid.",
            ));
            continue;
        };
        let call = PluginHookCall {
            plugin_id: hook.plugin_id.clone(),
            hook: hook.hook,
            target: hook.target.clone(),
            config,
            input: PluginHookInput::TransportMessage(transport.clone()),
        };
        match executor.execute(call) {
            Ok(PluginHookOutput::TransportMessageTransform(
                crate::TransportMessageTransform::Unchanged,
            )) => {}
            Ok(PluginHookOutput::TransportMessageTransform(
                crate::TransportMessageTransform::Replace(next),
            )) => transport = next,
            Ok(PluginHookOutput::TransportMessageTransform(
                crate::TransportMessageTransform::Drop { reason },
            )) => {
                diagnostics.push(message_diagnostic(
                    hook,
                    PluginDiagnosticSeverity::Warning,
                    &reason.unwrap_or_else(|| {
                        "Incoming message dropped by plugin transform.".to_owned()
                    }),
                ));
                return IncomingPluginResult {
                    event: None,
                    diagnostics,
                };
            }
            Ok(output) => diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Error,
                &format!("Unexpected plugin output: {output:?}"),
            )),
            Err(error) => diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Error,
                &error.to_string(),
            )),
        }
    }

    for hook in job
        .hooks
        .iter()
        .filter(|hook| hook.hook == PluginHookKind::Validator)
    {
        if cancelled.load(Ordering::Acquire) {
            diagnostics.push(cancellation_diagnostic());
            return IncomingPluginResult {
                event: None,
                diagnostics,
            };
        }
        let Ok(config) = serde_json::from_str(&hook.config_json) else {
            diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Error,
                "Plugin hook config is invalid.",
            ));
            continue;
        };
        let call = PluginHookCall {
            plugin_id: hook.plugin_id.clone(),
            hook: hook.hook,
            target: hook.target.clone(),
            config,
            input: PluginHookInput::TransportMessage(transport.clone()),
        };
        match executor.execute(call) {
            Ok(PluginHookOutput::Validation(PluginValidation::Valid)) => diagnostics.push(
                message_diagnostic(hook, PluginDiagnosticSeverity::Info, "Validation passed"),
            ),
            Ok(PluginHookOutput::Validation(PluginValidation::Warning { message })) => diagnostics
                .push(message_diagnostic(
                    hook,
                    PluginDiagnosticSeverity::Warning,
                    &message,
                )),
            Ok(PluginHookOutput::Validation(PluginValidation::Block { message })) => diagnostics
                .push(message_diagnostic(
                    hook,
                    PluginDiagnosticSeverity::Error,
                    &message,
                )),
            Ok(output) => diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Error,
                &format!("Unexpected plugin output: {output:?}"),
            )),
            Err(error) => diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Error,
                &error.to_string(),
            )),
        }
    }

    if cancelled.load(Ordering::Acquire) {
        diagnostics.push(cancellation_diagnostic());
        return IncomingPluginResult {
            event: None,
            diagnostics,
        };
    }

    let message = match plugin_message_from_transport(
        transport,
        retained_when_absent,
        &transport_capabilities,
    ) {
        Ok(message) => message,
        Err(error) => {
            diagnostics.push(MessageDiagnosticRow {
                severity: PluginDiagnosticSeverity::Error,
                hook: Some(PluginHookKind::IncomingTransform),
                plugin_id: None,
                message: error,
            });
            return IncomingPluginResult {
                event: None,
                diagnostics,
            };
        }
    };
    match incoming_from_plugin_message(job.message, message) {
        Ok(message) => IncomingPluginResult {
            event: Some(MqttEvent::IncomingMessage(message)),
            diagnostics,
        },
        Err(error) => {
            diagnostics.push(MessageDiagnosticRow {
                severity: PluginDiagnosticSeverity::Error,
                hook: Some(PluginHookKind::IncomingTransform),
                plugin_id: None,
                message: error,
            });
            IncomingPluginResult {
                event: None,
                diagnostics,
            }
        }
    }
}

fn cancellation_diagnostic() -> MessageDiagnosticRow {
    MessageDiagnosticRow {
        severity: PluginDiagnosticSeverity::Warning,
        hook: None,
        plugin_id: None,
        message: "Incoming plugin processing cancelled during shutdown.".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };

    use correo_mqtt::{ConnectionId, Qos, TopicName};

    use super::*;

    #[derive(Debug, Default)]
    struct BlockingExecutor {
        cancelled: AtomicBool,
        started: AtomicBool,
    }

    impl PluginHookExecutor for BlockingExecutor {
        fn execute(&self, _: PluginHookCall) -> Result<PluginHookOutput, crate::PluginHookError> {
            self.started.store(true, Ordering::Release);
            while !self.cancelled.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(crate::PluginHookError::cancelled("cancelled"))
        }

        fn cancel(&self) {
            self.cancelled.store(true, Ordering::Release);
        }
    }

    #[test]
    fn cancellation_interrupts_active_incoming_hook() {
        let executor = Arc::new(BlockingExecutor::default());
        let worker = IncomingPluginWorker::start(executor.clone());
        worker
            .enqueue(
                IncomingMessage {
                    connection_id: ConnectionId::new(),
                    topic: TopicName::new("bridge/cancelled").unwrap(),
                    payload: b"payload".to_vec(),
                    qos: Qos::AtMostOnce,
                    retain: false,
                    duplicate: false,
                    packet_id: None,
                },
                vec![ActiveHook {
                    plugin_id: "test.plugin".to_owned(),
                    hook: PluginHookKind::IncomingTransform,
                    target: "bridge/#".to_owned(),
                    config_json: "{}".to_owned(),
                }],
                Vec::new(),
            )
            .unwrap();

        let start_deadline = Instant::now() + Duration::from_secs(1);
        while !executor.started.load(Ordering::Acquire) {
            assert!(
                Instant::now() < start_deadline,
                "incoming hook did not start"
            );
            std::thread::sleep(Duration::from_millis(1));
        }

        worker.cancel();
        let deadline = Instant::now() + Duration::from_secs(1);
        let result = loop {
            if let Some(result) = worker.try_recv() {
                break result;
            }
            assert!(Instant::now() < deadline, "cancelled hook did not finish");
            std::thread::sleep(Duration::from_millis(1));
        };

        assert!(result.event.is_none());
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("cancelled")));
    }
}
