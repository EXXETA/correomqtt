use crate::{
    append_broker_log, AppCommand, BuiltInBrokerEvent, BuiltInBrokerProcessConfig,
    BuiltInBrokerStatus, Diagnostic,
};

use super::AppModel;

impl AppModel {
    pub(super) fn apply_broker_command(&mut self, command: &AppCommand) -> bool {
        match command {
            AppCommand::StartBuiltInBroker => self.request_broker_start(),
            AppCommand::StopBuiltInBroker => {
                self.snapshot.built_in_broker.status = BuiltInBrokerStatus::Stopping;
                append_broker_log(
                    &mut self.snapshot.built_in_broker.logs,
                    "Stopping broker...",
                );
            }
            AppCommand::UpdateBuiltInBrokerPort(port) => {
                self.snapshot.built_in_broker.port = port.clone();
                self.sync_built_in_broker_connection();
            }
            AppCommand::SetBuiltInBrokerCredentialsEnabled(enabled) => {
                self.snapshot.built_in_broker.credentials_enabled = *enabled;
                self.sync_built_in_broker_connection();
            }
            AppCommand::UpdateBuiltInBrokerUsername(username) => {
                self.snapshot.built_in_broker.username = username.clone();
                self.sync_built_in_broker_connection();
            }
            AppCommand::UpdateBuiltInBrokerPassword(password) => {
                self.snapshot.built_in_broker.password = password.expose_for_ui().to_owned();
                self.sync_built_in_broker_connection();
            }
            AppCommand::ClearBuiltInBrokerLogs => self.snapshot.built_in_broker.logs.clear(),
            _ => return false,
        }
        true
    }

    pub(super) fn apply_broker_event(&mut self, event: BuiltInBrokerEvent) {
        match event {
            BuiltInBrokerEvent::Started { port } => {
                self.snapshot.built_in_broker.status = BuiltInBrokerStatus::Running;
                self.sync_built_in_broker_connection();
                append_broker_log(
                    &mut self.snapshot.built_in_broker.logs,
                    format!("Built-in broker is listening on 127.0.0.1:{port}."),
                );
            }
            BuiltInBrokerEvent::Stopped { message } => {
                self.snapshot.built_in_broker.status = BuiltInBrokerStatus::Stopped;
                self.sync_built_in_broker_connection();
                append_broker_log(&mut self.snapshot.built_in_broker.logs, message);
            }
            BuiltInBrokerEvent::Failed { message } => {
                self.snapshot.built_in_broker.status = BuiltInBrokerStatus::Error;
                self.sync_built_in_broker_connection();
                append_broker_log(&mut self.snapshot.built_in_broker.logs, message.clone());
                self.push_diagnostic(Diagnostic::error(message));
            }
            BuiltInBrokerEvent::Log { message } => {
                append_broker_log(&mut self.snapshot.built_in_broker.logs, message);
            }
        }
    }

    pub(crate) fn broker_start_config(&self) -> Option<BuiltInBrokerProcessConfig> {
        let broker = &self.snapshot.built_in_broker;
        let port = broker
            .port
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|port| *port > 0)?;
        let (username, password) = if broker.credentials_enabled {
            let username = broker.username.trim();
            if username.is_empty() || broker.password.is_empty() {
                return None;
            }
            (Some(username.to_owned()), Some(broker.password.clone()))
        } else {
            (None, None)
        };
        Some(BuiltInBrokerProcessConfig {
            port,
            username,
            password,
        })
    }

    fn request_broker_start(&mut self) {
        if self.snapshot.built_in_broker.status.is_running() {
            append_broker_log(
                &mut self.snapshot.built_in_broker.logs,
                "Built-in broker is already running.",
            );
            return;
        }
        if self.broker_start_config().is_none() {
            self.snapshot.built_in_broker.status = BuiltInBrokerStatus::Error;
            append_broker_log(
                &mut self.snapshot.built_in_broker.logs,
                "Broker configuration is invalid. Use a port between 1 and 65535 with a username and password when credentials are enabled.",
            );
            return;
        }
        self.snapshot.built_in_broker.status = BuiltInBrokerStatus::Starting;
        append_broker_log(
            &mut self.snapshot.built_in_broker.logs,
            "Starting broker...",
        );
    }
}
