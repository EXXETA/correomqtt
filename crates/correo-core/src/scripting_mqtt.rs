use correo_mqtt::{ConnectionId, MqttError, Qos, Subscription, UnsubscribeRequest};
use std::sync::Arc;

use correo_scripting::{ScriptCancellationToken, ScriptMqttClient, ScriptPublishRequest};

use crate::{MqttCommand, MqttCommandSender, MqttServiceSendError};

pub(crate) struct ScriptMqttBridge {
    connection_id: Option<ConnectionId>,
    commands: MqttCommandSender,
    connect_command: Option<MqttCommand>,
}

impl ScriptMqttBridge {
    pub(crate) fn new(
        connection_id: Option<String>,
        commands: MqttCommandSender,
        connect_command: Option<MqttCommand>,
    ) -> Self {
        Self {
            connection_id: connection_id
                .and_then(|id| uuid::Uuid::parse_str(&id).ok())
                .map(ConnectionId::from_uuid),
            commands,
            connect_command,
        }
    }

    fn connection_id(&self) -> Result<ConnectionId, MqttError> {
        self.connection_id.ok_or_else(|| {
            MqttError::invalid_options("script MQTT calls require a selected connection")
        })
    }

    fn send(&self, command: MqttCommand) -> Result<(), MqttError> {
        self.commands.send(command).map_err(send_error)
    }

    fn check_cancelled(cancellation: &ScriptCancellationToken) -> Result<(), MqttError> {
        if cancellation.is_cancelled() {
            Err(MqttError::Cancelled)
        } else {
            Ok(())
        }
    }
}

pub(crate) fn client(
    connection_id: Option<String>,
    mqtt_sender: Option<MqttCommandSender>,
    connect_command: Option<MqttCommand>,
) -> Option<Arc<dyn ScriptMqttClient>> {
    mqtt_sender.map(|sender| {
        Arc::new(ScriptMqttBridge::new(
            connection_id,
            sender,
            connect_command,
        )) as _
    })
}

impl ScriptMqttClient for ScriptMqttBridge {
    fn connect(&self, cancellation: &ScriptCancellationToken) -> Result<(), MqttError> {
        Self::check_cancelled(cancellation)?;
        match self.connect_command.clone() {
            Some(command) => self.send(command),
            // No connect command is prepared when the connection is already
            // established: with a valid connection this is a successful no-op,
            // only a missing/invalid connection is an error.
            None => {
                self.connection_id()?;
                Ok(())
            }
        }
    }

    fn disconnect(&self, cancellation: &ScriptCancellationToken) -> Result<(), MqttError> {
        Self::check_cancelled(cancellation)?;
        self.send(MqttCommand::Disconnect {
            connection_id: self.connection_id()?,
        })
    }

    fn publish(
        &self,
        request: ScriptPublishRequest,
        cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        Self::check_cancelled(cancellation)?;
        self.send(MqttCommand::Publish {
            connection_id: self.connection_id()?,
            request: request.into_publish_request()?,
            diagnostics: Vec::new(),
        })
    }

    fn subscribe(
        &self,
        topic_filter: String,
        qos: Qos,
        cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        Self::check_cancelled(cancellation)?;
        let subscription = Subscription::new(topic_filter.as_str(), qos)?;
        self.send(MqttCommand::Subscribe {
            connection_id: self.connection_id()?,
            subscription,
        })
    }

    fn unsubscribe(
        &self,
        topic_filter: String,
        cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError> {
        Self::check_cancelled(cancellation)?;
        let request = UnsubscribeRequest::new(topic_filter.as_str())?;
        self.send(MqttCommand::Unsubscribe {
            connection_id: self.connection_id()?,
            request,
        })
    }
}

fn send_error(error: MqttServiceSendError) -> MqttError {
    MqttError::io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bridge(
        connection_id: Option<String>,
        connect_command: Option<MqttCommand>,
    ) -> (ScriptMqttBridge, flume::Receiver<MqttCommand>) {
        let (sender, receiver) = flume::unbounded();
        let bridge = ScriptMqttBridge::new(
            connection_id,
            MqttCommandSender::new(sender),
            connect_command,
        );
        (bridge, receiver)
    }

    #[test]
    fn connect_is_a_noop_when_already_connected() {
        // No prepared connect command means the connection is already up; with a
        // valid connection id this must succeed without sending a command.
        let (bridge, receiver) = bridge(Some(uuid::Uuid::new_v4().to_string()), None);
        assert!(bridge.connect(&ScriptCancellationToken::new()).is_ok());
        assert!(
            receiver.try_recv().is_err(),
            "already-connected connect() must not enqueue a command"
        );
    }

    #[test]
    fn connect_errors_without_a_connection() {
        let (bridge, _receiver) = bridge(None, None);
        assert!(bridge.connect(&ScriptCancellationToken::new()).is_err());
    }
}
