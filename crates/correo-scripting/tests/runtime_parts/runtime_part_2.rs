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
