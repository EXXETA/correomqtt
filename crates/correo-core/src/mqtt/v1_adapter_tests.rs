use serde_json::json;

use correo_mqtt::{ConnectionId, IncomingMessage, PublishRequest, Qos, TopicName};

use super::v1_adapter::{from_mqtt_command, from_mqtt_event};
use crate::{ConnectionCommand, DeliveryGuarantee, MqttCommand, MqttEvent, TransportEvent};

#[test]
fn transport_mqtt_v1_publish_and_incoming_event_keep_protocol_values_namespaced() {
    let connection_id = ConnectionId::new();
    let command = MqttCommand::Publish {
        connection_id,
        request: PublishRequest::new("orders.created", vec![1, 2, 3], Qos::AtLeastOnce, true)
            .unwrap(),
        diagnostics: Vec::new(),
    };

    let ConnectionCommand::Publish {
        connection_id: command_connection_id,
        message,
    } = from_mqtt_command(&command).expect("publish command is adapted")
    else {
        panic!("expected a transport publish command");
    };
    assert_eq!(command_connection_id.as_str(), connection_id.to_string());
    assert_eq!(message.delivery.guarantee, DeliveryGuarantee::AtLeastOnce);
    assert_eq!(
        message.metadata.get_value("mqtt.qos"),
        Some(&json!("at_least_once"))
    );
    assert_eq!(
        message.metadata.get_value("mqtt.retained"),
        Some(&json!(true))
    );

    let event = MqttEvent::IncomingMessage(IncomingMessage {
        connection_id,
        topic: TopicName::new("orders.created").unwrap(),
        payload: vec![1, 2, 3],
        qos: Qos::AtLeastOnce,
        retain: true,
        duplicate: false,
        packet_id: Some(7),
    });
    let TransportEvent::MessageReceived {
        connection_id: event_connection_id,
        message,
    } = from_mqtt_event(&event).expect("incoming MQTT event is adapted")
    else {
        panic!("expected a transport incoming-message event");
    };
    assert_eq!(event_connection_id.as_str(), connection_id.to_string());
    assert_eq!(
        message.metadata.get_value("mqtt.qos"),
        Some(&json!("at_least_once"))
    );
    assert_eq!(
        message.metadata.get_value("mqtt.retained"),
        Some(&json!(true))
    );
    assert_eq!(
        message.metadata.get_value("mqtt.duplicate"),
        Some(&json!(false))
    );
    assert_eq!(
        message.metadata.get_value("mqtt.packet_id"),
        Some(&json!(7))
    );
}
