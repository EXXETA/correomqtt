use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;
use rumqttc_v5 as rumqtt;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use super::common::{
    client_id, finish_startup, keep_alive_seconds, SessionChannels, StartupSignal,
};
use crate::{
    transport::{tls::rustls_client_config, PreparedTransport, TransportErrorReporter},
    ConnectionId, IncomingMessage, LastWill, MqttAuth, MqttConnectionOptions, MqttEndpoint,
    MqttError, MqttProtocolVersion, MqttResult, MqttSession, MqttSessionEvent, PublishRequest, Qos,
    SessionState, Subscription, TopicName, UnsubscribeRequest,
};

const CLIENT_ACK_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Mqtt5Session {
    channels: SessionChannels,
    client: Option<rumqtt::AsyncClient>,
    task: Option<JoinHandle<()>>,
    transport: Option<PreparedTransport>,
}

impl Mqtt5Session {
    pub fn new() -> Self {
        Self {
            channels: SessionChannels::new(),
            client: None,
            task: None,
            transport: None,
        }
    }

    async fn stop_existing(&mut self) {
        if let Some(client) = self.client.take() {
            let _ = client.disconnect().await;
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
        if let Some(mut transport) = self.transport.take() {
            let _ = transport.close().await;
        }
    }
}

impl Default for Mqtt5Session {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MqttSession for Mqtt5Session {
    async fn connect(&mut self, options: MqttConnectionOptions) -> MqttResult<()> {
        if options.protocol_version != MqttProtocolVersion::Mqtt5 {
            return Err(MqttError::invalid_options(
                "MQTT 5 session received non-5 options",
            ));
        }

        self.stop_existing().await;
        self.channels.set_state(SessionState::Connecting);

        let connection_id = options.connection_id;
        let error_channels = self.channels.clone();
        let error_reporter: TransportErrorReporter =
            Arc::new(move |error| error_channels.report_error(error));
        let mut transport =
            PreparedTransport::open_with_reporter(&options, Some(error_reporter)).await?;
        let mqtt_options = build_options(&options, &transport.endpoint)?;
        let (client, eventloop) = rumqtt::AsyncClient::builder(mqtt_options).build();
        let (startup_tx, startup_rx) = oneshot::channel();
        let channels = self.channels.clone();
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| MqttError::invalid_options("rumqtt sessions require a Tokio runtime"))?
            .spawn(run_loop(
                eventloop,
                channels,
                connection_id,
                Some(startup_tx),
            ));

        self.client = Some(client);
        self.task = Some(handle);

        let result = startup_rx
            .await
            .unwrap_or_else(|_| Err(MqttError::connect("MQTT event loop stopped during connect")));
        if result.is_err() {
            self.client = None;
            if let Some(task) = self.task.take() {
                task.abort();
            }
            let _ = transport.close().await;
        } else {
            self.transport = Some(transport);
        }
        result
    }

    async fn disconnect(&mut self) -> MqttResult<()> {
        self.channels.set_state(SessionState::Disconnecting);
        let Some(client) = self.client.take() else {
            if let Some(mut transport) = self.transport.take() {
                transport.close().await?;
            }
            self.channels.set_state(SessionState::Disconnected);
            return Ok(());
        };

        let disconnect_result = match timeout(CLIENT_ACK_TIMEOUT, client.disconnect()).await {
            Ok(result) => result.map_err(map_client_error),
            Err(_) => Err(MqttError::protocol("disconnect timed out")),
        };
        if let Some(mut task) = self.task.take() {
            if disconnect_result.is_ok() {
                if timeout(CLIENT_ACK_TIMEOUT, &mut task).await.is_err() {
                    task.abort();
                }
            } else {
                task.abort();
            }
        }
        let transport_result = if let Some(mut transport) = self.transport.take() {
            transport.close().await
        } else {
            Ok(())
        };
        self.channels.set_state(SessionState::Disconnected);
        disconnect_result?;
        transport_result?;
        Ok(())
    }

    async fn publish(&mut self, request: PublishRequest) -> MqttResult<()> {
        let client = self.client.as_ref().ok_or(MqttError::Disconnected)?;
        let notice = client
            .publish_tracked(
                request.topic.as_str(),
                to_qos(request.qos),
                request.retain,
                request.payload.clone(),
            )
            .await
            .map_err(map_client_error)?;
        wait_for_ack(notice.wait_completion_async(), "publish").await?;
        self.channels.report_published(MqttSessionEvent::Published {
            topic: request.topic,
            payload: request.payload,
            qos: request.qos,
            retain: request.retain,
        });
        Ok(())
    }

    async fn subscribe(&mut self, subscription: Subscription) -> MqttResult<()> {
        let client = self.client.as_ref().ok_or(MqttError::Disconnected)?;
        let notice = client
            .subscribe_tracked(subscription.topic_filter.as_str(), to_qos(subscription.qos))
            .await
            .map_err(map_client_error)?;
        wait_for_ack(notice.wait_completion_async(), "subscribe").await?;
        self.channels
            .report_published(MqttSessionEvent::Subscribed(subscription));
        Ok(())
    }

    async fn unsubscribe(&mut self, request: UnsubscribeRequest) -> MqttResult<()> {
        let client = self.client.as_ref().ok_or(MqttError::Disconnected)?;
        let notice = client
            .unsubscribe_tracked(request.topic_filter.as_str())
            .await
            .map_err(map_client_error)?;
        wait_for_ack(notice.wait_completion_async(), "unsubscribe").await?;
        self.channels
            .report_published(MqttSessionEvent::Unsubscribed(request));
        Ok(())
    }

    fn current_state(&self) -> SessionState {
        self.channels.current_state()
    }

    fn events(&mut self) -> BoxStream<'static, MqttSessionEvent> {
        self.channels.event_stream()
    }

    fn incoming(&mut self) -> BoxStream<'static, Result<IncomingMessage, MqttError>> {
        self.channels.incoming_stream()
    }
}

async fn wait_for_ack<F, E>(future: F, operation: &'static str) -> MqttResult<()>
where
    F: Future<Output = Result<(), E>>,
    E: std::fmt::Display,
{
    timeout(CLIENT_ACK_TIMEOUT, future)
        .await
        .map_err(|_| MqttError::protocol(format!("{operation} timed out")))?
        .map_err(|error| MqttError::protocol(error.to_string()))
}

async fn run_loop(
    mut eventloop: rumqtt::EventLoop,
    channels: SessionChannels,
    connection_id: ConnectionId,
    mut startup: Option<StartupSignal>,
) {
    let mut connected_once = false;
    let mut reconnect_attempt = 0;

    loop {
        match eventloop.poll().await {
            Ok(event) => handle_event(
                event,
                &channels,
                connection_id,
                &mut startup,
                &mut connected_once,
            ),
            Err(rumqtt::ConnectionError::RequestsDone) => {
                channels.set_state(SessionState::Disconnected);
                finish_startup(&mut startup, Err(MqttError::Disconnected));
                break;
            }
            Err(error) => {
                let terminal = is_terminal_error(&error);
                let mapped = map_connection_error(error);
                channels.report_error(mapped.clone());

                if !connected_once || terminal {
                    channels.set_state(SessionState::Faulted {
                        error: mapped.to_report(),
                    });
                    finish_startup(&mut startup, Err(mapped));
                    break;
                }

                reconnect_attempt += 1;
                channels.set_state(SessionState::Reconnecting {
                    attempt: reconnect_attempt,
                });
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

fn handle_event(
    event: rumqtt::Event,
    channels: &SessionChannels,
    connection_id: ConnectionId,
    startup: &mut Option<StartupSignal>,
    connected_once: &mut bool,
) {
    match event {
        rumqtt::Event::Incoming(rumqtt::Packet::ConnAck(_)) => {
            *connected_once = true;
            channels.set_state(SessionState::Connected);
            finish_startup(startup, Ok(()));
        }
        rumqtt::Event::Incoming(rumqtt::Packet::Publish(publish)) => {
            match incoming_from_publish(connection_id, publish) {
                Ok(message) => channels.report_incoming(message),
                Err(error) => channels.report_incoming_error(error),
            }
        }
        rumqtt::Event::Auth(_) | rumqtt::Event::Outgoing(_) | rumqtt::Event::Incoming(_) => {}
    }
}

fn build_options(
    options: &MqttConnectionOptions,
    endpoint: &MqttEndpoint,
) -> MqttResult<rumqtt::MqttOptions> {
    if options.protocol_version != MqttProtocolVersion::Mqtt5 {
        return Err(MqttError::invalid_options(
            "MQTT 5 session received non-5 options",
        ));
    }

    let mut mqtt_options = rumqtt::MqttOptions::new(
        client_id(options)?,
        rumqtt::Broker::tcp(endpoint.host.clone(), endpoint.port),
    );
    mqtt_options.set_clean_start(options.clean_start);
    mqtt_options.set_keep_alive(keep_alive_seconds(options.keep_alive)?);
    apply_auth(&mut mqtt_options, &options.auth);

    if let Some(will) = &options.last_will {
        mqtt_options.set_last_will(to_last_will(will));
    }
    apply_tls(&mut mqtt_options, options)?;

    Ok(mqtt_options)
}

fn apply_tls(
    mqtt_options: &mut rumqtt::MqttOptions,
    options: &MqttConnectionOptions,
) -> MqttResult<()> {
    if let Some(config) = rustls_client_config(&options.tls)? {
        mqtt_options.set_transport(rumqtt::Transport::tls_with_config(config.into()));
    }
    Ok(())
}

fn apply_auth(mqtt_options: &mut rumqtt::MqttOptions, auth: &MqttAuth) {
    match auth {
        MqttAuth::Anonymous => {}
        MqttAuth::UsernamePassword { username, password } => match username {
            Some(username) => {
                mqtt_options.set_credentials(
                    username.clone(),
                    password.expose_secret().as_bytes().to_vec(),
                );
            }
            None => {
                mqtt_options.set_auth(rumqtt::ConnectAuth::Password {
                    password: password.expose_secret().as_bytes().to_vec().into(),
                });
            }
        },
        MqttAuth::Token { token } => {
            mqtt_options.set_auth(rumqtt::ConnectAuth::Password {
                password: token.expose_secret().as_bytes().to_vec().into(),
            });
        }
    }
}

fn to_last_will(will: &LastWill) -> rumqtt::LastWill {
    rumqtt::LastWill::new(
        will.topic.as_str(),
        will.payload.clone(),
        to_qos(will.qos),
        will.retain,
        None,
    )
}

fn incoming_from_publish(
    connection_id: ConnectionId,
    publish: rumqtt::Publish,
) -> MqttResult<IncomingMessage> {
    let topic = String::from_utf8(publish.topic.to_vec())
        .map_err(|_| MqttError::protocol("incoming MQTT topic was not valid UTF-8"))?;
    Ok(IncomingMessage {
        connection_id,
        topic: TopicName::new(topic)?,
        payload: publish.payload.to_vec(),
        qos: from_qos(publish.qos),
        retain: publish.retain,
        duplicate: publish.dup,
        packet_id: (publish.pkid != 0).then_some(publish.pkid),
    })
}

fn to_qos(qos: Qos) -> rumqtt::QoS {
    match qos {
        Qos::AtMostOnce => rumqtt::QoS::AtMostOnce,
        Qos::AtLeastOnce => rumqtt::QoS::AtLeastOnce,
        Qos::ExactlyOnce => rumqtt::QoS::ExactlyOnce,
    }
}

fn from_qos(qos: rumqtt::QoS) -> Qos {
    match qos {
        rumqtt::QoS::AtMostOnce => Qos::AtMostOnce,
        rumqtt::QoS::AtLeastOnce => Qos::AtLeastOnce,
        rumqtt::QoS::ExactlyOnce => Qos::ExactlyOnce,
    }
}

fn map_client_error(error: rumqtt::ClientError) -> MqttError {
    MqttError::protocol(error.to_string())
}

fn map_connection_error(error: rumqtt::ConnectionError) -> MqttError {
    match error {
        rumqtt::ConnectionError::ConnectionRefused(code) => map_return_code(code),
        rumqtt::ConnectionError::MqttState(rumqtt::StateError::ConnFail { reason }) => {
            map_return_code(reason)
        }
        rumqtt::ConnectionError::MqttState(rumqtt::StateError::ServerDisconnect {
            reason_code,
            reason_string,
        }) => MqttError::protocol(format!(
            "server disconnect: {reason_code:?} {reason_string:?}"
        )),
        rumqtt::ConnectionError::Io(error) => MqttError::io(error.to_string()),
        rumqtt::ConnectionError::Timeout(_) => MqttError::connect("network timeout"),
        rumqtt::ConnectionError::DisconnectTimeout => MqttError::connect("disconnect timeout"),
        other => MqttError::connect(other.to_string()),
    }
}

fn map_return_code(code: rumqtt::ConnectReturnCode) -> MqttError {
    match code {
        rumqtt::ConnectReturnCode::BadUserNamePassword
        | rumqtt::ConnectReturnCode::NotAuthorized
        | rumqtt::ConnectReturnCode::BadAuthenticationMethod => {
            MqttError::auth(format!("{code:?}"))
        }
        _ => MqttError::connect(format!("{code:?}")),
    }
}

fn is_terminal_error(error: &rumqtt::ConnectionError) -> bool {
    matches!(
        error,
        rumqtt::ConnectionError::ConnectionRefused(
            rumqtt::ConnectReturnCode::BadClientId
                | rumqtt::ConnectReturnCode::BadUserNamePassword
                | rumqtt::ConnectReturnCode::NotAuthorized
                | rumqtt::ConnectReturnCode::BadAuthenticationMethod
                | rumqtt::ConnectReturnCode::ClientIdentifierNotValid
                | rumqtt::ConnectReturnCode::MalformedPacket
                | rumqtt::ConnectReturnCode::ProtocolError
                | rumqtt::ConnectReturnCode::RefusedProtocolVersion
                | rumqtt::ConnectReturnCode::UnsupportedProtocolVersion
        ) | rumqtt::ConnectionError::MqttState(rumqtt::StateError::ConnFail {
            reason: rumqtt::ConnectReturnCode::BadClientId
                | rumqtt::ConnectReturnCode::BadUserNamePassword
                | rumqtt::ConnectReturnCode::NotAuthorized
                | rumqtt::ConnectReturnCode::BadAuthenticationMethod
                | rumqtt::ConnectReturnCode::ClientIdentifierNotValid
                | rumqtt::ConnectReturnCode::MalformedPacket
                | rumqtt::ConnectReturnCode::ProtocolError
                | rumqtt::ConnectReturnCode::RefusedProtocolVersion
                | rumqtt::ConnectReturnCode::UnsupportedProtocolVersion
        })
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MqttEndpoint, SecretString};

    fn options(protocol: MqttProtocolVersion) -> MqttConnectionOptions {
        let mut options = MqttConnectionOptions::new(
            ConnectionId::new(),
            "local",
            MqttEndpoint::new("localhost", 1883).expect("valid endpoint"),
        );
        options.protocol_version = protocol;
        options
    }

    #[test]
    fn build_options_rejects_non_v5_protocol() {
        let options = options(MqttProtocolVersion::Mqtt3_1_1);
        let error = build_options(&options, &options.endpoint).expect_err("invalid");
        assert!(matches!(error, MqttError::InvalidOptions { .. }));
    }

    #[test]
    fn v5_accepts_password_without_username() {
        let mut options = options(MqttProtocolVersion::Mqtt5);
        options.auth = MqttAuth::UsernamePassword {
            username: None,
            password: SecretString::new("synthetic-password"),
        };

        let mqtt_options = build_options(&options, &options.endpoint).expect("valid");
        match mqtt_options.auth() {
            rumqtt::ConnectAuth::Password { password } => {
                assert_eq!(password.as_ref(), b"synthetic-password");
            }
            other => panic!("unexpected auth variant: {other:?}"),
        }
    }

    #[test]
    fn incoming_publish_maps_packet_fields() {
        let connection_id = ConnectionId::new();
        let mut publish = rumqtt::Publish::new(
            "devices/alpha/state",
            rumqtt::QoS::AtLeastOnce,
            b"online".to_vec(),
            None,
        );
        publish.retain = true;
        publish.pkid = 7;

        let message = incoming_from_publish(connection_id, publish).expect("mapped");
        assert_eq!(message.connection_id, connection_id);
        assert_eq!(message.topic.as_str(), "devices/alpha/state");
        assert_eq!(message.payload, b"online");
        assert_eq!(message.qos, Qos::AtLeastOnce);
        assert!(message.retain);
        assert!(!message.duplicate);
        assert_eq!(message.packet_id, Some(7));
    }
}
