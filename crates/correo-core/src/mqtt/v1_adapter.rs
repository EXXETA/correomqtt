use serde_json::json;

use correo_mqtt::{IncomingMessage, PublishRequest, Qos};

#[cfg(test)]
use crate::{ConnectionCommand, MqttCommand, MqttEvent, TransportConnectionId, TransportEvent};
use crate::{DeliveryGuarantee, DeliverySemantics, MessageEnvelope, NamespacedName};

#[cfg(test)]
pub(super) fn from_mqtt_command(command: &MqttCommand) -> Option<ConnectionCommand> {
    let MqttCommand::Publish {
        connection_id,
        request,
        ..
    } = command
    else {
        return None;
    };

    Some(ConnectionCommand::Publish {
        connection_id: transport_connection_id(connection_id.to_string()),
        message: message_from_publish(request),
    })
}

#[cfg(test)]
pub(super) fn from_mqtt_event(event: &MqttEvent) -> Option<TransportEvent> {
    let MqttEvent::IncomingMessage(message) = event else {
        return None;
    };

    Some(TransportEvent::MessageReceived {
        connection_id: transport_connection_id(message.connection_id.to_string()),
        message: message_from_incoming(message),
    })
}

pub(crate) fn message_from_incoming(incoming: &IncomingMessage) -> MessageEnvelope {
    let mut message = message_from_parts(
        incoming.topic.to_string(),
        incoming.payload.clone(),
        incoming.qos,
        incoming.retain,
    );
    message
        .metadata
        .insert(metadata_key("mqtt.duplicate"), json!(incoming.duplicate));
    if let Some(packet_id) = incoming.packet_id {
        message
            .metadata
            .insert(metadata_key("mqtt.packet_id"), json!(packet_id));
    }
    message
}

pub(crate) fn message_from_publish(request: &PublishRequest) -> MessageEnvelope {
    message_from_parts(
        request.topic.to_string(),
        request.payload.clone(),
        request.qos,
        request.retain,
    )
}

fn message_from_parts(address: String, body: Vec<u8>, qos: Qos, retained: bool) -> MessageEnvelope {
    let mut message = MessageEnvelope::new(address, body, DeliverySemantics::new(delivery(qos)));
    message
        .metadata
        .insert(metadata_key("mqtt.qos"), json!(qos_label(qos)));
    message
        .metadata
        .insert(metadata_key("mqtt.retained"), json!(retained));
    message
}

#[cfg(test)]
fn transport_connection_id(value: String) -> TransportConnectionId {
    TransportConnectionId::new(value).expect("MQTT connection IDs are never empty")
}

fn metadata_key(value: &'static str) -> NamespacedName {
    NamespacedName::new(value).expect("MQTT metadata keys are namespaced")
}

fn delivery(qos: Qos) -> DeliveryGuarantee {
    match qos {
        Qos::AtMostOnce => DeliveryGuarantee::AtMostOnce,
        Qos::AtLeastOnce => DeliveryGuarantee::AtLeastOnce,
        Qos::ExactlyOnce => DeliveryGuarantee::ExactlyOnce,
    }
}

fn qos_label(qos: Qos) -> &'static str {
    match qos {
        Qos::AtMostOnce => "at_most_once",
        Qos::AtLeastOnce => "at_least_once",
        Qos::ExactlyOnce => "exactly_once",
    }
}
