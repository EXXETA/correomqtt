use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};


use correo_mqtt::{MqttError, Qos};
use rquickjs::{context::intrinsic, CatchResultExt, CaughtError, Context, Object, Persistent, Promise, Runtime};
use time::OffsetDateTime;

use crate::{
    bindings::install_bindings, NoopScriptHost, ScriptCancellationToken, ScriptExecutionId,
    ScriptExecutionMetadata, ScriptExecutionStatus, ScriptHost, ScriptLogEntry, ScriptLogLevel,
    ScriptPublishRequest, ScriptingError, ScriptingResult,
};

const SCRIPT_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;
const SCRIPT_STACK_LIMIT_BYTES: usize = 1024 * 1024;
const SCRIPT_GC_THRESHOLD_BYTES: usize = 4 * 1024 * 1024;
const SCRIPT_DEFAULT_DEADLINE: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub struct ScriptRuntime {
    host: Arc<dyn ScriptHost>,
}
impl ScriptRuntime {
    pub fn new(host: Arc<dyn ScriptHost>) -> Self {
        Self { host }
    }

    pub fn execute(
        &self,
        request: ScriptExecutionRequest,
        cancellation: ScriptCancellationToken,
    ) -> ScriptExecutionOutcome {
        let mut metadata = ScriptExecutionMetadata {
            id: request.id,
            script_name: request.script_name,
            started_at: OffsetDateTime::now_utc(),
            status: ScriptExecutionStatus::Running,
        };
        self.host.execution_metadata_changed(&metadata);

        let error = self.run_source(&request.source, &cancellation).err();
        metadata.status = match error {
            None => ScriptExecutionStatus::Succeeded,
            Some(ScriptingError::Cancelled) => ScriptExecutionStatus::Cancelled,
            Some(_) => ScriptExecutionStatus::Failed,
        };
        self.host.execution_metadata_changed(&metadata);

        ScriptExecutionOutcome {
            metadata,
            finished_at: OffsetDateTime::now_utc(),
            error,
        }
    }

    fn run_source(
        &self,
        source: &str,
        cancellation: &ScriptCancellationToken,
    ) -> ScriptingResult<()> {
        if cancellation.is_cancelled() {
            return Err(ScriptingError::Cancelled);
        }

        let runtime = Runtime::new().map_err(|error| ScriptingError::Runtime(error.to_string()))?;
        runtime.set_memory_limit(SCRIPT_MEMORY_LIMIT_BYTES);
        runtime.set_max_stack_size(SCRIPT_STACK_LIMIT_BYTES);
        runtime.set_gc_threshold(SCRIPT_GC_THRESHOLD_BYTES);
        let interrupt_token = cancellation.clone();
        let deadline_cancelled = Arc::new(AtomicBool::new(false));
        let interrupt_deadline_cancelled = deadline_cancelled.clone();
        let deadline = Instant::now() + SCRIPT_DEFAULT_DEADLINE;
        runtime.set_interrupt_handler(Some(Box::new(move || {
            if interrupt_token.is_cancelled() {
                return true;
            }
            if Instant::now() < deadline {
                return false;
            }
            if !interrupt_deadline_cancelled.swap(true, Ordering::SeqCst) {
                interrupt_token.cancel();
            }
            true
        })));

        let context = Context::builder()
            .with::<intrinsic::Eval>()
            .with::<intrinsic::Json>()
            .with::<intrinsic::Promise>()
            .build(&runtime)
            .map_err(|error| ScriptingError::Runtime(error.to_string()))?;

        let state = Arc::new(HostState::new(self.host.clone(), cancellation.clone()));
        context.with(|ctx| {
            let result = (|| {
                install_bindings(ctx.clone(), state.clone())
                    .map_err(|error| ScriptingError::Runtime(error.to_string()))?;

                if source.contains("await") {
                    let promise = ctx
                        .eval::<Promise<'_>, _>(async_wrapped_source(source))
                        .catch(&ctx)
                        .map_err(|error| state.map_guest_error(error).unwrap_err())?;
                    drain_pending_jobs(&ctx, &state)?;
                    match promise.result::<()>() {
                        Some(result) => result
                            .catch(&ctx)
                            .map_err(|error| state.map_guest_error(error).unwrap_err()),
                        None => Ok(()),
                    }
                } else {
                    match ctx.eval::<(), _>(source).catch(&ctx) {
                        Ok(()) => drain_pending_jobs(&ctx, &state),
                        Err(error) => state.map_guest_error(error),
                    }
                }
            })();
            state.clear_promise_host_errors();
            result
        })
    }
}

fn async_wrapped_source(source: &str) -> String {
    format!("(async () => {{\n{source}\n}})();")
}

fn drain_pending_jobs(ctx: &rquickjs::Ctx<'_>, state: &HostState) -> ScriptingResult<()> {
    for _ in 0..10_000 {
        state.check_cancelled()?;
        if !ctx.execute_pending_job() {
            return Ok(());
        }
    }
    Err(ScriptingError::Runtime(
        "JavaScript promise queue did not settle".to_owned(),
    ))
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new(Arc::new(NoopScriptHost))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptExecutionRequest {
    pub id: ScriptExecutionId,
    pub script_name: String,
    pub source: String,
}
impl ScriptExecutionRequest {
    pub fn new(script_name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            id: ScriptExecutionId::new(),
            script_name: script_name.into(),
            source: source.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptExecutionOutcome {
    pub metadata: ScriptExecutionMetadata,
    pub finished_at: OffsetDateTime,
    pub error: Option<ScriptingError>,
}
impl ScriptExecutionOutcome {
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}

struct PromiseHostError {
    exception: Persistent<Object<'static>>,
    error: ScriptingError,
}

pub(crate) struct HostState {
    host: Arc<dyn ScriptHost>,
    cancellation: ScriptCancellationToken,
    last_host_error: Mutex<Option<ScriptingError>>,
    queue_processing: AtomicBool,
    promise_host_errors: Mutex<Vec<PromiseHostError>>,
}
impl HostState {
    fn new(host: Arc<dyn ScriptHost>, cancellation: ScriptCancellationToken) -> Self {
        Self {
            host,
            cancellation,
            last_host_error: Mutex::new(None),
            queue_processing: AtomicBool::new(true),
            promise_host_errors: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn check_cancelled(&self) -> ScriptingResult<()> {
        if self.cancellation.is_cancelled() {
            Err(ScriptingError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub(crate) fn sleep(&self, millis: i32) -> ScriptingResult<()> {
        if millis < 0 {
            return Err(ScriptingError::HostApi(
                "sleep(ms) requires a non-negative duration".to_owned(),
            ));
        }

        let deadline = Duration::from_millis(millis as u64);
        let step = Duration::from_millis(10);
        let mut slept = Duration::ZERO;
        while slept < deadline {
            self.check_cancelled()?;
            let current = deadline.saturating_sub(slept).min(step);
            thread::sleep(current);
            slept += current;
        }

        self.check_cancelled()
    }

    pub(crate) fn log(&self, level: ScriptLogLevel, message: String) -> ScriptingResult<()> {
        self.check_cancelled()?;
        self.host.log(ScriptLogEntry::new(level, message));
        Ok(())
    }

    pub(crate) fn connect(&self) -> ScriptingResult<()> {
        self.check_cancelled()?;
        let client = self.mqtt_client()?;
        if let Some(handle) = client.cancellation_handle() {
            self.cancellation.register_handle(handle);
        }
        match client.connect(&self.cancellation) {
            Ok(()) => self.check_cancelled(),
            Err(_) if self.cancellation.is_cancelled() => Err(ScriptingError::Cancelled),
            Err(error) => Err(Self::mqtt_error(error)),
        }
    }

    pub(crate) fn disconnect(&self) -> ScriptingResult<()> {
        self.check_cancelled()?;
        let client = self.mqtt_client()?;
        if let Some(handle) = client.cancellation_handle() {
            self.cancellation.register_handle(handle);
        }
        match client.disconnect(&self.cancellation) {
            Ok(()) => self.check_cancelled(),
            Err(_) if self.cancellation.is_cancelled() => Err(ScriptingError::Cancelled),
            Err(error) => Err(Self::mqtt_error(error)),
        }
    }

    pub(crate) fn publish(
        &self,
        topic: String,
        payload: String,
        qos: Qos,
        retain: bool,
    ) -> ScriptingResult<()> {
        self.check_cancelled()?;
        let client = self.mqtt_client()?;
        let request = ScriptPublishRequest {
            topic,
            payload: payload.into_bytes(),
            qos,
            retain,
        };

        match client.publish(request, &self.cancellation) {
            Ok(()) => self.check_cancelled(),
            Err(_) if self.cancellation.is_cancelled() => Err(ScriptingError::Cancelled),
            Err(error) => Err(Self::mqtt_error(error)),
        }
    }

    pub(crate) fn subscribe(&self, topic_filter: String, qos: Qos) -> ScriptingResult<()> {
        self.check_cancelled()?;
        let client = self.mqtt_client()?;
        match client.subscribe(topic_filter, qos, &self.cancellation) {
            Ok(()) => self.check_cancelled(),
            Err(_) if self.cancellation.is_cancelled() => Err(ScriptingError::Cancelled),
            Err(error) => Err(Self::mqtt_error(error)),
        }
    }

    pub(crate) fn unsubscribe(&self, topic_filter: String) -> ScriptingResult<()> {
        self.check_cancelled()?;
        let client = self.mqtt_client()?;
        match client.unsubscribe(topic_filter, &self.cancellation) {
            Ok(()) => self.check_cancelled(),
            Err(_) if self.cancellation.is_cancelled() => Err(ScriptingError::Cancelled),
            Err(error) => Err(Self::mqtt_error(error)),
        }
    }

    pub(crate) fn queue_process(&self) -> ScriptingResult<bool> {
        self.check_cancelled()?;
        Ok(self.queue_processing.load(Ordering::SeqCst))
    }

    pub(crate) fn queue_jump_out(&self) -> ScriptingResult<()> {
        self.check_cancelled()?;
        self.queue_processing.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn mqtt_client(&self) -> ScriptingResult<Arc<dyn crate::ScriptMqttClient>> {
        let client = self.host.mqtt_client().ok_or_else(|| {
            ScriptingError::HostApi("MQTT client is not available to this script".to_owned())
        })?;

        if let Some(handle) = client.cancellation_handle() {
            self.cancellation.register_handle(handle);
        }

        Ok(client)
    }

    fn map_guest_error(&self, error: CaughtError<'_>) -> ScriptingResult<()> {
        if self.cancellation.is_cancelled() {
            self.cancellation.cancel_owned_operations();
            return Err(ScriptingError::Cancelled);
        }

        if let CaughtError::Exception(exception) = &error {
            if let Some(error) = self.promise_host_error(exception) {
                return Err(error);
            }
        }

        if let Some(error) = self
            .last_host_error
            .lock()
            .expect("script host error lock poisoned")
            .take()
        {
            return Err(error);
        }
        Err(ScriptingError::JavaScriptGuest(error.to_string()))
    }

    pub(crate) fn register_promise_host_error<'js>(
        &self,
        ctx: &rquickjs::Ctx<'js>,
        exception: &rquickjs::Exception<'js>,
        error: &ScriptingError,
    ) {
        self.promise_host_errors
            .lock()
            .expect("promise host error lock poisoned")
            .push(PromiseHostError {
                exception: Persistent::save(ctx, exception.as_object().clone()),
                error: error.clone(),
            });
    }

    fn clear_promise_host_errors(&self) {
        self.promise_host_errors
            .lock()
            .expect("promise host error lock poisoned")
            .clear();
    }

    fn promise_host_error(&self, exception: &rquickjs::Exception<'_>) -> Option<ScriptingError> {
        let host_errors = self
            .promise_host_errors
            .lock()
            .expect("promise host error lock poisoned");
        host_errors.iter().find_map(|host_error| {
            let registered = host_error.exception.clone().restore(exception.as_object().ctx()).ok()?;
            if registered == *exception.as_object() {
                Some(host_error.error.clone())
            } else {
                None
            }
        })
    }


    pub(crate) fn throw_host_error(&self, error: ScriptingError) -> rquickjs::Error {
        *self
            .last_host_error
            .lock()
            .expect("script host error lock poisoned") = Some(error.clone());

        rquickjs::Error::new_from_js_message("host", "CorreoMQTT script host", error.to_string())
    }

    fn mqtt_error(error: MqttError) -> ScriptingError {
        ScriptingError::MqttOperation(error.to_string())
    }
}

