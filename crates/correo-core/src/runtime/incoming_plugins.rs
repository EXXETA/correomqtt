use std::sync::Arc;

use correo_mqtt::IncomingMessage;
use flume::{Receiver, Sender};

use crate::{
    MessageDiagnosticRow, MqttEvent, PluginDiagnosticSeverity, PluginHookCall, PluginHookExecutor,
    PluginHookInput, PluginHookKind, PluginHookOutput, PluginValidation,
};

use super::plugin_helpers::{
    incoming_from_plugin_message, message_diagnostic, plugin_message_from_incoming, ActiveHook,
};

const INCOMING_PLUGIN_CAPACITY: usize = 128;

#[derive(Debug)]
pub(super) struct IncomingPluginWorker {
    requests: Sender<IncomingPluginJob>,
    results: Receiver<IncomingPluginResult>,
}

impl IncomingPluginWorker {
    pub(super) fn start(executor: Arc<dyn PluginHookExecutor>) -> Self {
        Self::start_with_capacity(executor, INCOMING_PLUGIN_CAPACITY)
    }

    #[cfg(test)]
    pub(super) fn start_with_capacity(
        executor: Arc<dyn PluginHookExecutor>,
        capacity: usize,
    ) -> Self {
        Self::start_with_capacity_inner(executor, capacity)
    }

    #[cfg(not(test))]
    fn start_with_capacity(executor: Arc<dyn PluginHookExecutor>, capacity: usize) -> Self {
        Self::start_with_capacity_inner(executor, capacity)
    }

    fn start_with_capacity_inner(executor: Arc<dyn PluginHookExecutor>, capacity: usize) -> Self {
        let (requests, request_receiver) = flume::bounded(capacity);
        let (result_sender, results) = flume::bounded(capacity);
        std::thread::Builder::new()
            .name("correo-plugin-ingress".to_owned())
            .spawn(move || {
                while let Ok(job) = request_receiver.recv() {
                    let result = run_job(job, executor.as_ref());
                    if result_sender.send(result).is_err() {
                        break;
                    }
                }
            })
            .expect("plugin ingress worker thread can start");
        Self { requests, results }
    }

    pub(super) fn enqueue(
        &self,
        message: IncomingMessage,
        transforms: Vec<ActiveHook>,
        validators: Vec<ActiveHook>,
        diagnostics: Vec<MessageDiagnosticRow>,
    ) -> Result<(), IncomingPluginQueueError> {
        self.requests
            .try_send(IncomingPluginJob {
                message,
                transforms,
                validators,
                diagnostics,
            })
            .map_err(|error| match error {
                flume::TrySendError::Full(job) => IncomingPluginQueueError::Full(job),
                flume::TrySendError::Disconnected(job) => {
                    IncomingPluginQueueError::Disconnected(job)
                }
            })
    }

    pub(super) fn try_recv(&self) -> Option<IncomingPluginResult> {
        self.results.try_recv().ok()
    }
}

#[derive(Debug)]
pub(super) enum IncomingPluginQueueError {
    Full(IncomingPluginJob),
    Disconnected(IncomingPluginJob),
}

pub(super) struct IncomingPluginResult {
    pub(super) event: Option<MqttEvent>,
    pub(super) diagnostics: Vec<MessageDiagnosticRow>,
}

#[derive(Debug)]
pub(super) struct IncomingPluginJob {
    pub(super) message: IncomingMessage,
    transforms: Vec<ActiveHook>,
    validators: Vec<ActiveHook>,
    pub(super) diagnostics: Vec<MessageDiagnosticRow>,
}

impl IncomingPluginQueueError {
    pub(super) fn into_job(self) -> IncomingPluginJob {
        match self {
            Self::Full(job) | Self::Disconnected(job) => job,
        }
    }

    pub(super) fn detail(&self) -> &'static str {
        match self {
            Self::Full(_) => "Incoming plugin queue is full",
            Self::Disconnected(_) => "Incoming plugin worker is unavailable",
        }
    }
}

fn run_job(job: IncomingPluginJob, executor: &dyn PluginHookExecutor) -> IncomingPluginResult {
    let mut diagnostics = job.diagnostics;
    let mut plugin_message = plugin_message_from_incoming(&job.message);

    for hook in job.transforms {
        if !execute_transform(&hook, &mut plugin_message, executor, &mut diagnostics) {
            return IncomingPluginResult {
                event: None,
                diagnostics,
            };
        }
    }
    for hook in job.validators {
        execute_validator(&hook, &plugin_message, executor, &mut diagnostics);
    }

    match incoming_from_plugin_message(job.message, plugin_message) {
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

fn execute_transform(
    hook: &ActiveHook,
    message: &mut crate::PluginMessage,
    executor: &dyn PluginHookExecutor,
    diagnostics: &mut Vec<MessageDiagnosticRow>,
) -> bool {
    let Ok(config) = serde_json::from_str(&hook.config_json) else {
        diagnostics.push(message_diagnostic(
            hook,
            PluginDiagnosticSeverity::Error,
            "Plugin hook config is invalid.",
        ));
        return true;
    };
    let call = PluginHookCall {
        plugin_id: hook.plugin_id.clone(),
        hook: hook.hook,
        target: hook.target.clone(),
        config,
        input: PluginHookInput::Message(message.clone()),
    };
    match executor.execute(call) {
        Ok(PluginHookOutput::MessageTransform(crate::MessageTransform::Unchanged)) => true,
        Ok(PluginHookOutput::MessageTransform(crate::MessageTransform::Replace(next))) => {
            *message = next;
            true
        }
        Ok(PluginHookOutput::MessageTransform(crate::MessageTransform::Drop { reason })) => {
            diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Warning,
                &reason
                    .unwrap_or_else(|| "Incoming message dropped by plugin transform.".to_owned()),
            ));
            false
        }
        Ok(output) => {
            diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Error,
                &format!("Unexpected plugin output: {output:?}"),
            ));
            true
        }
        Err(error) => {
            diagnostics.push(message_diagnostic(
                hook,
                PluginDiagnosticSeverity::Error,
                &error.to_string(),
            ));
            true
        }
    }
}

fn execute_validator(
    hook: &ActiveHook,
    message: &crate::PluginMessage,
    executor: &dyn PluginHookExecutor,
    diagnostics: &mut Vec<MessageDiagnosticRow>,
) {
    let Ok(config) = serde_json::from_str(&hook.config_json) else {
        diagnostics.push(message_diagnostic(
            hook,
            PluginDiagnosticSeverity::Error,
            "Plugin hook config is invalid.",
        ));
        return;
    };
    let call = PluginHookCall {
        plugin_id: hook.plugin_id.clone(),
        hook: hook.hook,
        target: hook.target.clone(),
        config,
        input: PluginHookInput::Message(message.clone()),
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
        Ok(PluginHookOutput::Validation(PluginValidation::Block { message })) => diagnostics.push(
            message_diagnostic(hook, PluginDiagnosticSeverity::Error, &message),
        ),
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
