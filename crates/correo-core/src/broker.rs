use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::thread;

use serde::{Deserialize, Serialize};
use time::{macros::format_description, OffsetDateTime};

use correo_mqtt::ConnectionId;
use uuid::Uuid;

use crate::{
    AppEvent, AppEventSender, ConnectDisabledReason, ConnectionBadge, ConnectionSettingsSnapshot,
    ConnectionState, ConnectionSummary, KeyringState, SecretInput,
};

const CHILD_ARG: &str = "--correo-builtin-broker-child";
const MAX_LOG_ENTRIES: usize = 200;
pub const BUILT_IN_BROKER_CONNECTION_NAME: &str = "Correo Broker";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltInBrokerSnapshot {
    pub port: String,
    pub credentials_enabled: bool,
    pub username: String,
    pub password: String,
    pub status: BuiltInBrokerStatus,
    pub logs: Vec<BuiltInBrokerLogEntry>,
}

impl fmt::Debug for BuiltInBrokerSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltInBrokerSnapshot")
            .field("port", &self.port)
            .field("credentials_enabled", &self.credentials_enabled)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("status", &self.status)
            .field("logs", &self.logs)
            .finish()
    }
}

impl Default for BuiltInBrokerSnapshot {
    fn default() -> Self {
        Self {
            port: "1883".to_owned(),
            credentials_enabled: false,
            username: String::new(),
            password: String::new(),
            status: BuiltInBrokerStatus::Stopped,
            logs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltInBrokerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl BuiltInBrokerStatus {
    pub fn is_running(self) -> bool {
        matches!(self, Self::Starting | Self::Running | Self::Stopping)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltInBrokerLogEntry {
    pub timestamp: String,
    pub message: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltInBrokerProcessConfig {
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl fmt::Debug for BuiltInBrokerProcessConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltInBrokerProcessConfig")
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltInBrokerEvent {
    Started { port: u16 },
    Stopped { message: String },
    Failed { message: String },
    Log { message: String },
}

#[derive(Debug)]
pub struct BuiltInBrokerWorker {
    event_sender: AppEventSender,
    child: Option<Child>,
}

impl BuiltInBrokerWorker {
    pub fn new(event_sender: AppEventSender) -> Self {
        Self {
            event_sender,
            child: None,
        }
    }

    pub fn start(&mut self, config: BuiltInBrokerProcessConfig) {
        self.reap_finished_child();
        if self.child.is_some() {
            self.emit(BuiltInBrokerEvent::Log {
                message: "Built-in broker is already running.".to_owned(),
            });
            return;
        }

        match self.spawn_child(config.clone()) {
            Ok(child) => {
                self.child = Some(child);
                self.emit(BuiltInBrokerEvent::Started { port: config.port });
            }
            Err(error) => self.emit(BuiltInBrokerEvent::Failed { message: error }),
        }
    }

    pub fn stop(&mut self) {
        let Some(mut child) = self.child.take() else {
            self.emit(BuiltInBrokerEvent::Stopped {
                message: "Built-in broker is not running.".to_owned(),
            });
            return;
        };

        let result = child.kill().and_then(|()| child.wait().map(|_| ()));
        match result {
            Ok(()) => self.emit(BuiltInBrokerEvent::Stopped {
                message: "Built-in broker stopped.".to_owned(),
            }),
            Err(error) => self.emit(BuiltInBrokerEvent::Failed {
                message: format!("Built-in broker could not be stopped: {error}"),
            }),
        }
    }

    pub fn poll(&mut self) {
        self.reap_finished_child();
    }

    fn spawn_child(&self, config: BuiltInBrokerProcessConfig) -> Result<Child, String> {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut child = Command::new(executable)
            .arg(CHILD_ARG)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("Built-in broker could not be started: {error}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            let json = serde_json::to_vec(&config).map_err(|error| error.to_string())?;
            stdin.write_all(&json).map_err(|error| error.to_string())?;
        }
        if let Some(stdout) = child.stdout.take() {
            spawn_log_reader(stdout, self.event_sender.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_log_reader(stderr, self.event_sender.clone());
        }
        Ok(child)
    }

    fn reap_finished_child(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        match child.try_wait() {
            Ok(Some(status)) => {
                self.child = None;
                self.emit(BuiltInBrokerEvent::Stopped {
                    message: format!("Built-in broker exited with {status}."),
                });
            }
            Ok(None) => {}
            Err(error) => {
                self.child = None;
                self.emit(BuiltInBrokerEvent::Failed {
                    message: format!("Built-in broker status could not be checked: {error}"),
                });
            }
        }
    }

    fn emit(&self, event: BuiltInBrokerEvent) {
        let _ = self.event_sender.emit(AppEvent::BuiltInBroker(event));
    }
}

impl Drop for BuiltInBrokerWorker {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn builtin_broker_child_arg() -> &'static str {
    CHILD_ARG
}

pub fn built_in_broker_connection_id() -> ConnectionId {
    ConnectionId::from_uuid(Uuid::from_u128(0xc011_e0b0_0000_4000_8000_000000000001))
}

pub fn is_built_in_broker_connection(id: ConnectionId) -> bool {
    id == built_in_broker_connection_id()
}

pub fn built_in_broker_connection_summary(broker: &BuiltInBrokerSnapshot) -> ConnectionSummary {
    let settings = built_in_broker_connection_settings(broker);
    let credentials = broker
        .credentials_enabled
        .then_some(ConnectionBadge::Credentials);
    ConnectionSummary {
        id: built_in_broker_connection_id(),
        name: BUILT_IN_BROKER_CONNECTION_NAME.to_owned(),
        endpoint: format!("{}:{}", settings.host, settings.port),
        mqtt_version: settings.mqtt_version.clone(),
        badges: credentials.into_iter().collect(),
        active_plugin_workflows: false,
        immutable: true,
        state: ConnectionState::Disconnected,
        disabled_reason: if broker.status == BuiltInBrokerStatus::Running {
            None
        } else {
            Some(ConnectDisabledReason::BrokerStopped)
        },
        recent_subscriptions: 0,
        recent_messages: 0,
        last_activity: "Ready".to_owned(),
    }
}

pub fn built_in_broker_connection_settings(
    broker: &BuiltInBrokerSnapshot,
) -> ConnectionSettingsSnapshot {
    let mut settings = ConnectionSettingsSnapshot {
        internal_id: built_in_broker_connection_id().to_string(),
        profile_name: BUILT_IN_BROKER_CONNECTION_NAME.to_owned(),
        host: "127.0.0.1".to_owned(),
        port: broker.port.clone(),
        mqtt_version: "MQTT v5".to_owned(),
        clean_session: true,
        client_id: "correomqtt-broker-client".to_owned(),
        username: if broker.credentials_enabled {
            broker.username.clone()
        } else {
            String::new()
        },
        password: if broker.credentials_enabled {
            SecretInput::new(broker.password.clone())
        } else {
            SecretInput::default()
        },
        password_status: if broker.credentials_enabled {
            "MQTT password configured for built-in broker".to_owned()
        } else {
            "No MQTT password configured".to_owned()
        },
        tls_mode: "No TLS/SSL".to_owned(),
        tls_host_verification: true,
        proxy_mode: "No proxy/tunnel".to_owned(),
        ssh_port: "22".to_owned(),
        local_mqtt_port: broker.port.clone(),
        auth_mode: "No Auth".to_owned(),
        tls_password_status: "No SSL password configured".to_owned(),
        ssh_password_status: "No SSH password configured".to_owned(),
        lwt_retained: false,
        dirty: false,
        valid: true,
        save_disabled_reason: "Built-in broker connection is managed automatically".to_owned(),
        keyring_state: KeyringState::Available,
        ..ConnectionSettingsSnapshot::default()
    };
    if settings
        .port
        .trim()
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .is_none()
    {
        settings.valid = false;
        settings.save_disabled_reason = "Built-in broker port is invalid".to_owned();
        settings
            .validation_errors
            .push("Port must be between 1 and 65535".to_owned());
    }
    settings
}

pub(crate) fn append_broker_log(logs: &mut Vec<BuiltInBrokerLogEntry>, message: impl Into<String>) {
    logs.insert(
        0,
        BuiltInBrokerLogEntry {
            timestamp: timestamp(),
            message: message.into(),
        },
    );
    logs.truncate(MAX_LOG_ENTRIES);
}

fn spawn_log_reader(stream: impl std::io::Read + Send + 'static, sender: AppEventSender) {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines().map_while(Result::ok) {
            let message = line.trim().to_owned();
            if !message.is_empty() {
                let _ = sender.emit(AppEvent::BuiltInBroker(BuiltInBrokerEvent::Log { message }));
            }
        }
    });
}

fn timestamp() -> String {
    const FORMAT: &[time::format_description::FormatItem<'_>] =
        format_description!("[hour]:[minute]:[second]");
    OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .format(FORMAT)
        .unwrap_or_else(|_| "--:--:--".to_owned())
}
