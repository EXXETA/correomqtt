mod common;
mod v3;
mod v5;

const MAX_MQTT_PACKET_SIZE_BYTES: u32 = 10 * 1024 * 1024;

use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::{
    IncomingMessage, MqttConnectionOptions, MqttError, MqttProtocolVersion, MqttResult,
    MqttSession, MqttSessionEvent, PublishRequest, SessionState, Subscription, UnsubscribeRequest,
};

pub use v3::Mqtt311Session;
pub use v5::Mqtt5Session;

pub enum RumqttSession {
    Mqtt3(Mqtt311Session),
    Mqtt5(Mqtt5Session),
}

impl RumqttSession {
    pub fn for_protocol(protocol: MqttProtocolVersion) -> Self {
        match protocol {
            MqttProtocolVersion::Mqtt3_1_1 => Self::Mqtt3(Mqtt311Session::new()),
            MqttProtocolVersion::Mqtt5 => Self::Mqtt5(Mqtt5Session::new()),
        }
    }
}

#[async_trait]
impl MqttSession for RumqttSession {
    async fn connect(&mut self, options: MqttConnectionOptions) -> MqttResult<()> {
        match self {
            Self::Mqtt3(session) => session.connect(options).await,
            Self::Mqtt5(session) => session.connect(options).await,
        }
    }

    async fn disconnect(&mut self) -> MqttResult<()> {
        match self {
            Self::Mqtt3(session) => session.disconnect().await,
            Self::Mqtt5(session) => session.disconnect().await,
        }
    }

    async fn publish(&mut self, request: PublishRequest) -> MqttResult<()> {
        match self {
            Self::Mqtt3(session) => session.publish(request).await,
            Self::Mqtt5(session) => session.publish(request).await,
        }
    }

    async fn subscribe(&mut self, subscription: Subscription) -> MqttResult<()> {
        match self {
            Self::Mqtt3(session) => session.subscribe(subscription).await,
            Self::Mqtt5(session) => session.subscribe(subscription).await,
        }
    }

    async fn unsubscribe(&mut self, request: UnsubscribeRequest) -> MqttResult<()> {
        match self {
            Self::Mqtt3(session) => session.unsubscribe(request).await,
            Self::Mqtt5(session) => session.unsubscribe(request).await,
        }
    }

    fn current_state(&self) -> SessionState {
        match self {
            Self::Mqtt3(session) => session.current_state(),
            Self::Mqtt5(session) => session.current_state(),
        }
    }

    fn events(&mut self) -> BoxStream<'static, MqttSessionEvent> {
        match self {
            Self::Mqtt3(session) => session.events(),
            Self::Mqtt5(session) => session.events(),
        }
    }

    fn incoming(&mut self) -> BoxStream<'static, Result<IncomingMessage, MqttError>> {
        match self {
            Self::Mqtt3(session) => session.incoming(),
            Self::Mqtt5(session) => session.incoming(),
        }
    }
}
