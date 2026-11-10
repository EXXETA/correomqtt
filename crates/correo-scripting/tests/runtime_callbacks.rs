use std::sync::{Arc, Mutex};

use correo_mqtt::{MqttError, Qos};
use correo_scripting::{
    ScriptCancellationToken, ScriptExecutionRequest, ScriptHost, ScriptLogEntry, ScriptMqttClient,
    ScriptPublishRequest, ScriptRuntime, ScriptingError,
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
fn async_client_dispatches_script_publish_to_matching_subscribe_callback() {
    let mqtt = Arc::new(RecordingMqttClient::default());
    let source = r##"
        var client = clientFactory.getAsyncClient();

        client.connect(() => {
            client.subscribe("/step1", 1, (msg) => {
                logger.info("{}", 1);
                client.publish("/step2", 1, "foo2");
            });

            client.subscribe("/step2", 1, (msg) => {
                logger.info("{}", msg);
                queue.jumpOut();
            });

            client.publish("/step1", 1, "foo1", () => {
                queue.process();
                client.unsubscribeAll();
                client.disconnect();
                logger.info("Finished");
            });
        });
    "##;

    let (host, error) = execute_with_mqtt(source, mqtt.clone());

    assert_eq!(error, None);
    let logs = host
        .logs()
        .into_iter()
        .map(|log| log.message)
        .collect::<Vec<_>>();
    assert_eq!(logs, ["1", "foo2", "Finished"]);
    assert_eq!(
        mqtt.publishes
            .lock()
            .expect("publishes lock poisoned")
            .len(),
        2
    );
}

#[test]
fn blocking_client_supports_legacy_incoming_message_and_base64_helpers() {
    let mqtt = Arc::new(RecordingMqttClient::default());
    let source = r##"
        var client = clientFactory.getBlockingClient();

        client.connect();
        logger.info("{}", plugins.base64.decode("Zm9v"));

        client.subscribe("/step1", 1, (msg) => {
            logger.info("{}", 1);
            client.publish("/step2", 1, "Zm9vMg==");
        });

        client.onIncomingMessage("/step2", (payload) => {
            return plugins.base64.decode(payload);
        });

        client.subscribe("/step2", 1, (msg) => {
            logger.info("{}", msg);
        });

        client.publish("/step1", 1, "foo1");
        queue.process();
        client.unsubscribeAll();
        client.disconnect();
        logger.info("Finished");
    "##;

    let (host, error) = execute_with_mqtt(source, mqtt.clone());

    assert_eq!(error, None);
    let logs = host
        .logs()
        .into_iter()
        .map(|log| log.message)
        .collect::<Vec<_>>();
    assert_eq!(logs, ["foo", "1", "foo2", "Finished"]);
    assert_eq!(
        mqtt.publishes
            .lock()
            .expect("publishes lock poisoned")
            .len(),
        2
    );
}

#[test]
fn promise_client_supports_direct_promise_subscription_callbacks() {
    let mqtt = Arc::new(RecordingMqttClient::default());
    let source = r##"
        var client = new ClientFactory().getPromiseClient();

        client.connect().then(() => {
            client.subscribe("/step1", 1, (msg) => {
                logger.info("{}", 1);
                client.publish("/step2", 1, "foo2");
            });

            client.subscribe("/step2", 1, (msg) => {
                logger.info("{}", msg);
                queue.jumpOut();
            });

            client.publish("/step1", 1, "foo1").then(() => {
                queue.process();
                client.unsubscribeAll();
                client.disconnect();
                logger.info("Finished");
            });
        });
    "##;

    let (host, error) = execute_with_mqtt(source, mqtt.clone());

    assert_eq!(error, None);
    let logs = host
        .logs()
        .into_iter()
        .map(|log| log.message)
        .collect::<Vec<_>>();
    assert_eq!(logs, ["1", "foo2", "Finished"]);
    assert_eq!(
        mqtt.publishes
            .lock()
            .expect("publishes lock poisoned")
            .len(),
        2
    );
}

#[test]
fn mixed_client_supports_top_level_await_and_mode_conversion() {
    let mqtt = Arc::new(RecordingMqttClient::default());
    let source = r##"
        var client = new ClientFactory().getBlockingClient();

        client.connect();
        client = client.toPromised();

        await client.subscribe("/step1", 1, (msg) => {
            logger.info("{}", 1);
            client.publish("/step2", 1, "foo2");
        });

        await client.subscribe("/step2", 1, (msg) => {
            logger.info("{}", msg);
            queue.jumpOut();
        });

        await client.publish("/step1", 1, "foo1").then(() => {
            queue.process();
            client.unsubscribeAll();
            client.disconnect();
            logger.info("Finished");
        });

        client = client.toBlocking();
        client.unsubscribeAll();
        client.disconnect();
    "##;

    let (host, error) = execute_with_mqtt(source, mqtt.clone());

    assert_eq!(error, None);
    let logs = host
        .logs()
        .into_iter()
        .map(|log| log.message)
        .collect::<Vec<_>>();
    assert_eq!(logs, ["1", "foo2", "Finished"]);
    assert_eq!(
        mqtt.publishes
            .lock()
            .expect("publishes lock poisoned")
            .len(),
        2
    );
}

#[test]
fn denied_host_surfaces_are_absent() {
    let source = r#"
        if (typeof Java !== "undefined") throw new Error("Java exposed");
        if (typeof require !== "undefined") throw new Error("require exposed");
        if (typeof process !== "undefined") throw new Error("process exposed");
        if (typeof fetch !== "undefined") throw new Error("fetch exposed");
        if (typeof Deno !== "undefined") throw new Error("Deno exposed");
        if (typeof std !== "undefined") throw new Error("std exposed");
        if (typeof os !== "undefined") throw new Error("os exposed");
    "#;

    let (_, error) = execute(source);

    assert_eq!(error, None);
}
