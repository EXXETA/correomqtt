use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use correo_mqtt::{MqttError, Qos};
use correo_scripting::{
    ScriptCancellationHandle, ScriptCancellationToken, ScriptExecutionRequest, ScriptHost,
    ScriptLogEntry, ScriptLogLevel, ScriptMqttClient, ScriptPublishRequest, ScriptRuntime,
    ScriptingError,
};

#[derive(Default)]
struct TestHost {
    logs: Mutex<Vec<ScriptLogEntry>>,
    mqtt: Mutex<Option<Arc<dyn ScriptMqttClient>>>,
}

impl TestHost {
    fn with_mqtt(mqtt: Arc<dyn ScriptMqttClient>) -> Self {
        Self {
            logs: Mutex::new(Vec::new()),
            mqtt: Mutex::new(Some(mqtt)),
        }
    }

    fn logs(&self) -> Vec<ScriptLogEntry> {
        self.logs.lock().expect("logs lock poisoned").clone()
    }
}

impl ScriptHost for TestHost {
    fn log(&self, entry: ScriptLogEntry) {
        self.logs.lock().expect("logs lock poisoned").push(entry);
    }

    fn mqtt_client(&self) -> Option<Arc<dyn ScriptMqttClient>> {
        self.mqtt.lock().expect("mqtt lock poisoned").clone()
    }
}

fn execute(source: &str) -> (Arc<TestHost>, Option<ScriptingError>) {
    let host = Arc::new(TestHost::default());
    let runtime = ScriptRuntime::new(host.clone());
    let outcome = runtime.execute(
        ScriptExecutionRequest::new("test.js", source),
        ScriptCancellationToken::new(),
    );
    (host, outcome.error)
}

fn execute_with_mqtt(
    source: &str,
    mqtt: Arc<dyn ScriptMqttClient>,
) -> (Arc<TestHost>, Option<ScriptingError>) {
    let host = Arc::new(TestHost::with_mqtt(mqtt));
    let runtime = ScriptRuntime::new(host.clone());
    let outcome = runtime.execute(
        ScriptExecutionRequest::new("mqtt.js", source),
        ScriptCancellationToken::new(),
    );
    (host, outcome.error)
}

#[derive(Default)]
struct RecordingMqttClient {
    publishes: Mutex<Vec<ScriptPublishRequest>>,
    subscribes: Mutex<Vec<(String, Qos)>>,
    unsubscribes: Mutex<Vec<String>>,
}

impl ScriptMqttClient for RecordingMqttClient {
    fn publish(
        &self,
        request: ScriptPublishRequest,
        _cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        self.publishes
            .lock()
            .expect("publishes lock poisoned")
            .push(request);
        Ok(())
    }

    fn subscribe(
        &self,
        topic_filter: String,
        qos: Qos,
        _cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        self.subscribes
            .lock()
            .expect("subscribes lock poisoned")
            .push((topic_filter, qos));
        Ok(())
    }

    fn unsubscribe(
        &self,
        topic_filter: String,
        _cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        self.unsubscribes
            .lock()
            .expect("unsubscribes lock poisoned")
            .push(topic_filter);
        Ok(())
    }
}

#[test]
fn compatibility_aliases_are_available() {
    let source = r#"
        function assert(condition, message) {
            if (!condition) throw new Error(message);
        }

        const blocking = clientFactory.getBlockingClient();
        const asyncClient = clientFactory.getAsyncClient();
        const promiseClient = clientFactory.getPromiseClient();
        let connected = false;

        assert(typeof blocking.connect === "function", "blocking connect missing");
        assert(typeof blocking.publish === "function", "blocking publish missing");
        assert(typeof asyncClient.connect === "function", "async connect missing");
        assert(typeof asyncClient.subscribe === "function", "async subscribe missing");
        assert(typeof asyncClient.disconnect === "function", "async disconnect missing");
        assert(typeof asyncClient.unsubscribeAll === "function", "async unsubscribeAll missing");
        assert(typeof promiseClient.unsubscribe === "function", "promise unsubscribe missing");
        asyncClient.connect(() => { connected = true; });
        assert(connected === true, "async connect callback missing");
        assert(typeof sleep === "function", "sleep missing");
        assert(typeof join === "function", "join missing");
        assert(queue.process() === true, "queue should initially process");
        queue.jumpOut();
        assert(queue.process() === false, "queue jumpOut should stop processing");
        sleep(0);
        join();
        logger.info("aliases ok");
    "#;

    // connect() needs a host MQTT client since scripts drive real connections.
    let (host, error) = execute_with_mqtt(source, Arc::new(RecordingMqttClient::default()));

    assert_eq!(error, None);
    let logs = host.logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, ScriptLogLevel::Info);
    assert_eq!(logs[0].message, "aliases ok");
}

#[test]
fn blocking_client_accepts_modern_and_legacy_pubsub_shapes() {
    let mqtt = Arc::new(RecordingMqttClient::default());
    let source = r#"
        const client = clientFactory.getBlockingClient();
        client.publish("topic/block/default", 0);
        client.publish("topic/block/legacy", 1, "legacy payload");
        client.publish("topic/block/modern", "modern payload", { qos: 2, retain: true });
        client.subscribe("topic/block/+", 1, function (_payload) {});
        client.unsubscribe("topic/block/+");
    "#;

    let (_, error) = execute_with_mqtt(source, mqtt.clone());

    assert_eq!(error, None);
    let publishes = mqtt.publishes.lock().expect("publishes lock poisoned");
    assert_eq!(publishes.len(), 3);
    assert_eq!(publishes[0].topic, "topic/block/default");
    assert_eq!(publishes[0].payload, b"");
    assert_eq!(publishes[0].qos, Qos::AtMostOnce);
    assert_eq!(publishes[1].payload, b"legacy payload");
    assert_eq!(publishes[1].qos, Qos::AtLeastOnce);
    assert_eq!(publishes[2].payload, b"modern payload");
    assert_eq!(publishes[2].qos, Qos::ExactlyOnce);
    assert!(publishes[2].retain);
    drop(publishes);

    assert_eq!(
        mqtt.subscribes
            .lock()
            .expect("subscribes lock poisoned")
            .as_slice(),
        &[("topic/block/+".to_owned(), Qos::AtLeastOnce)]
    );
    assert_eq!(
        mqtt.unsubscribes
            .lock()
            .expect("unsubscribes lock poisoned")
            .as_slice(),
        &["topic/block/+".to_owned()]
    );
}

#[test]
fn async_client_accepts_legacy_pubsub_callbacks() {
    let mqtt = Arc::new(RecordingMqttClient::default());
    let source = r#"
        let resolved = 0;
        let rejected = 0;
        const client = clientFactory.getAsyncClient();
        const onSuccess = function () { resolved += 1; };
        const onError = function () { rejected += 1; };

        client.publish("topic/async/pub", 1, "async payload", onSuccess, onError);
        client.subscribe("topic/async/+", 2, onSuccess, onError, function (_payload) {});
        client.unsubscribe("topic/async/+", onSuccess, onError);

        if (resolved !== 3) throw new Error("async callbacks did not resolve");
        if (rejected !== 0) throw new Error("async callbacks rejected unexpectedly");
    "#;

    let (_, error) = execute_with_mqtt(source, mqtt.clone());

    assert_eq!(error, None);
    assert_eq!(
        mqtt.publishes
            .lock()
            .expect("publishes lock poisoned")
            .len(),
        1
    );
    assert_eq!(
        mqtt.subscribes
            .lock()
            .expect("subscribes lock poisoned")
            .as_slice(),
        &[("topic/async/+".to_owned(), Qos::ExactlyOnce)]
    );
    assert_eq!(
        mqtt.unsubscribes
            .lock()
            .expect("unsubscribes lock poisoned")
            .as_slice(),
        &["topic/async/+".to_owned()]
    );
}

#[test]
fn promise_client_executes_pubsub_when_awaited() {
    let mqtt = Arc::new(RecordingMqttClient::default());
    let source = r#"
        const client = clientFactory.getPromiseClient();

        await client.publish("topic/promise/default", "promise payload");
        await client.publish("topic/promise/legacy", 1, "legacy promise payload");
        await client.subscribe("topic/promise/+", 1, function (_payload) {});
        await client.unsubscribe("topic/promise/+");
    "#;

    let (_, error) = execute_with_mqtt(source, mqtt.clone());

    assert_eq!(error, None);
    let publishes = mqtt.publishes.lock().expect("publishes lock poisoned");
    assert_eq!(publishes.len(), 2);
    assert_eq!(publishes[0].payload, b"promise payload");
    assert_eq!(publishes[0].qos, Qos::AtMostOnce);
    assert_eq!(publishes[1].payload, b"legacy promise payload");
    assert_eq!(publishes[1].qos, Qos::AtLeastOnce);
    drop(publishes);
    assert_eq!(
        mqtt.subscribes
            .lock()
            .expect("subscribes lock poisoned")
            .as_slice(),
        &[("topic/promise/+".to_owned(), Qos::AtLeastOnce)]
    );
    assert_eq!(
        mqtt.unsubscribes
            .lock()
            .expect("unsubscribes lock poisoned")
            .as_slice(),
        &["topic/promise/+".to_owned()]
    );
}

#[test]
fn javascript_guest_errors_are_typed() {
    let (_, error) = execute(r#"throw new Error("guest boom");"#);

    assert!(matches!(
        error,
        Some(ScriptingError::JavaScriptGuest(message)) if message.contains("guest boom")
    ));
}

#[test]
fn host_api_errors_are_typed() {
    let (_, error) = execute("sleep(-1);");

    assert!(matches!(error, Some(ScriptingError::HostApi(message))
        if message.contains("non-negative")));
}

#[derive(Default)]
struct FailingMqttClient {
    requests: Mutex<Vec<ScriptPublishRequest>>,
}

impl ScriptMqttClient for FailingMqttClient {
    fn publish(
        &self,
        request: ScriptPublishRequest,
        _cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .push(request);
        Err(MqttError::Disconnected)
    }

    fn subscribe(
        &self,
        _topic_filter: String,
        _qos: Qos,
        _cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        Ok(())
    }

    fn unsubscribe(
        &self,
        _topic_filter: String,
        _cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        Ok(())
    }
}

#[test]
fn mqtt_errors_are_typed_and_requests_are_narrow() {
    let mqtt = Arc::new(FailingMqttClient::default());
    let host = Arc::new(TestHost::with_mqtt(mqtt.clone()));
    let runtime = ScriptRuntime::new(host);
    let outcome = runtime.execute(
        ScriptExecutionRequest::new(
            "mqtt.js",
            r#"clientFactory.getBlockingClient().publish("topic/test", "payload", { qos: 1, retain: true });"#,
        ),
        ScriptCancellationToken::new(),
    );

    assert!(matches!(
        outcome.error,
        Some(ScriptingError::MqttOperation(_))
    ));
    let requests = mqtt.requests.lock().expect("requests lock poisoned");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].topic, "topic/test");
    assert_eq!(requests[0].payload, b"payload");
    assert_eq!(requests[0].qos, Qos::AtLeastOnce);
    assert!(requests[0].retain);
}

#[test]
fn handled_promise_mqtt_error_does_not_poison_later_guest_error() {
    let mqtt = Arc::new(FailingMqttClient::default());
    let host = Arc::new(TestHost::with_mqtt(mqtt));
    let runtime = ScriptRuntime::new(host);
    let outcome = runtime.execute(
        ScriptExecutionRequest::new(
            "promise-error.js",
            r#"
            const client = clientFactory.getPromiseClient();
            try {
                await client.publish("topic/test", "payload");
            } catch (_error) {
                throw new Error("guest failure after handled promise rejection");
            }
            "#,
        ),
        ScriptCancellationToken::new(),
    );

    assert!(matches!(
        outcome.error,
        Some(ScriptingError::JavaScriptGuest(message))
            if message.contains("guest failure after handled promise rejection")
    ));
}

#[test]
fn handled_promise_mqtt_error_message_rethrow_stays_guest_error() {
    let mqtt = Arc::new(FailingMqttClient::default());
    let host = Arc::new(TestHost::with_mqtt(mqtt));
    let runtime = ScriptRuntime::new(host);
    let outcome = runtime.execute(
        ScriptExecutionRequest::new(
            "promise-error-rethrow.js",
            r#"
            const client = clientFactory.getPromiseClient();
            try {
                await client.publish("topic/test", "payload");
            } catch (error) {
                throw new Error(error.message);
            }
            "#,
        ),
        ScriptCancellationToken::new(),
    );

    assert!(matches!(
        outcome.error,
        Some(ScriptingError::JavaScriptGuest(message))
            if message.contains("script MQTT operation failed")
    ));
}

#[test]
fn handled_promise_mqtt_error_property_copy_rethrow_stays_guest_error() {
    let mqtt = Arc::new(FailingMqttClient::default());
    let host = Arc::new(TestHost::with_mqtt(mqtt));
    let runtime = ScriptRuntime::new(host);
    let outcome = runtime.execute(
        ScriptExecutionRequest::new(
            "promise-error-property-copy-rethrow.js",
            r#"
            const client = clientFactory.getPromiseClient();
            try {
                await client.publish("topic/test", "payload");
            } catch (error) {
                const rethrown = new Error(error.message);
                for (const property of Object.getOwnPropertyNames(error)) {
                    rethrown[property] = error[property];
                }
                throw rethrown;
            }
            "#,
        ),
        ScriptCancellationToken::new(),
    );

    assert!(matches!(
        outcome.error,
        Some(ScriptingError::JavaScriptGuest(message))
            if message.contains("script MQTT operation failed")
    ));
}
#[test]
fn awaited_promise_mqtt_error_stays_typed() {
    let mqtt = Arc::new(FailingMqttClient::default());
    let host = Arc::new(TestHost::with_mqtt(mqtt));
    let runtime = ScriptRuntime::new(host);
    let outcome = runtime.execute(
        ScriptExecutionRequest::new(
            "promise-mqtt-error.js",
            r#"
            const client = clientFactory.getPromiseClient();
            await client.publish("topic/test", "payload");
            "#,
        ),
        ScriptCancellationToken::new(),
    );

    assert!(matches!(
        outcome.error,
        Some(ScriptingError::MqttOperation(_))
    ));
}

#[test]
fn guest_error_containing_static_promise_marker_stays_guest_error() {
    let runtime = ScriptRuntime::default();
    let outcome = runtime.execute(
        ScriptExecutionRequest::new(
            "spoof-marker.js",
            r#"throw new Error("not internal __correomqtt_host_error__:mqtt:spoofed");"#,
        ),
        ScriptCancellationToken::new(),
    );

    assert!(matches!(
        outcome.error,
        Some(ScriptingError::JavaScriptGuest(message))
            if message.contains("__correomqtt_host_error__:mqtt:spoofed")
    ));
}

#[test]
fn guest_error_with_static_promise_marker_prefix_stays_guest_error() {
    let runtime = ScriptRuntime::default();
    let outcome = runtime.execute(
        ScriptExecutionRequest::new(
            "spoof-prefix-marker.js",
            r#"throw new Error("__correomqtt_host_error__:mqtt:spoofed");"#,
        ),
        ScriptCancellationToken::new(),
    );

    assert!(matches!(
        outcome.error,
        Some(ScriptingError::JavaScriptGuest(message))
            if message.contains("__correomqtt_host_error__:mqtt:spoofed")
    ));
}

#[test]
fn cancellation_interrupts_tight_javascript_loop() {
    let runtime = ScriptRuntime::default();
    let cancellation = ScriptCancellationToken::new();
    let thread_cancellation = cancellation.clone();
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let outcome = runtime.execute(
            ScriptExecutionRequest::new("loop.js", "while (true) {}"),
            thread_cancellation,
        );
        sender.send(outcome).expect("send cancellation outcome");
    });

    thread::sleep(Duration::from_millis(25));
    cancellation.cancel();
    let outcome = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("script should stop after cancellation");

    assert_eq!(outcome.error, Some(ScriptingError::Cancelled));
}

#[test]
fn deadline_cancels_tight_javascript_loop() {
    let runtime = ScriptRuntime::default();
    let cancellation = ScriptCancellationToken::new();
    let outcome = runtime.execute_with_timeout(
        ScriptExecutionRequest::new("deadline-loop.js", "while (true) {}"),
        cancellation.clone(),
        Duration::from_millis(50),
    );

    assert_eq!(outcome.error, Some(ScriptingError::Cancelled));
    assert!(cancellation.is_cancelled());
}

#[test]
fn deadline_interrupts_host_sleep() {
    let runtime = ScriptRuntime::default();
    let cancellation = ScriptCancellationToken::new();
    let started = Instant::now();
    let outcome = runtime.execute_with_timeout(
        ScriptExecutionRequest::new("deadline-sleep.js", "sleep(60000);"),
        cancellation.clone(),
        Duration::from_millis(50),
    );

    assert_eq!(outcome.error, Some(ScriptingError::Cancelled));
    assert!(cancellation.is_cancelled());
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[derive(Default)]
struct CountingCancelHandle {
    calls: AtomicUsize,
}

impl ScriptCancellationHandle for CountingCancelHandle {
    fn cancel(&self) {
        self.calls.fetch_add(1, Ordering::SeqCst);
    }
}

struct BlockingMqttClient {
    started: Mutex<Option<mpsc::Sender<()>>>,
    cancel_handle: Arc<CountingCancelHandle>,
}

impl ScriptMqttClient for BlockingMqttClient {
    fn publish(
        &self,
        _request: ScriptPublishRequest,
        cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        if let Some(sender) = self.started.lock().expect("started lock poisoned").take() {
            sender.send(()).expect("send publish started");
        }
        while !cancellation.is_cancelled() {
            thread::sleep(Duration::from_millis(5));
        }
        Err(MqttError::Cancelled)
    }

    fn subscribe(
        &self,
        _topic_filter: String,
        _qos: Qos,
        _cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        Ok(())
    }

    fn unsubscribe(
        &self,
        _topic_filter: String,
        _cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        Ok(())
    }

    fn cancellation_handle(&self) -> Option<Arc<dyn ScriptCancellationHandle>> {
        Some(self.cancel_handle.clone())
    }
}

#[test]
fn cancellation_cancels_owned_mqtt_operation() {
    let (started_sender, started_receiver) = mpsc::channel();
    let cancel_handle = Arc::new(CountingCancelHandle::default());
    let mqtt = Arc::new(BlockingMqttClient {
        started: Mutex::new(Some(started_sender)),
        cancel_handle: cancel_handle.clone(),
    });
    let runtime = ScriptRuntime::new(Arc::new(TestHost::with_mqtt(mqtt)));
    let cancellation = ScriptCancellationToken::new();
    let thread_cancellation = cancellation.clone();
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let outcome = runtime.execute(
            ScriptExecutionRequest::new(
                "mqtt-cancel.js",
                r#"clientFactory.getBlockingClient().publish("topic/test", "payload");"#,
            ),
            thread_cancellation,
        );
        sender
            .send(outcome)
            .expect("send mqtt cancellation outcome");
    });

    started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("publish should start");
    cancellation.cancel();
    let outcome = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("script should stop after cancelling MQTT operation");

    assert_eq!(outcome.error, Some(ScriptingError::Cancelled));
    assert!(cancel_handle.calls.load(Ordering::SeqCst) >= 1);
}
