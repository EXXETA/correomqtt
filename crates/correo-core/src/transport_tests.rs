use serde_json::json;

use crate::{
    ConnectionCommand, ConsumerSelection, DeliveryGuarantee, DeliverySemantics, MessageEnvelope,
    NamespacedName, ProtocolMetadata, TransportCapabilities, TransportCapability,
    TransportConnectionId,
};

#[test]
fn protocol_metadata_and_capabilities_require_namespaces() {
    assert!(NamespacedName::new("qos").is_err());

    let capability = TransportCapability::new("mqtt.retained").unwrap();
    let capabilities = TransportCapabilities::from([capability.clone()]);
    assert!(capabilities.contains(&capability));

    let key = NamespacedName::new("mqtt.qos").unwrap();
    let mut metadata = ProtocolMetadata::default();
    metadata.insert(key.clone(), json!("at_least_once"));
    assert_eq!(metadata.get(&key), Some(&json!("at_least_once")));
}

#[test]
fn transport_commands_carry_generic_message_consumer_and_delivery_values() {
    let connection_id = TransportConnectionId::new("connection-a").unwrap();
    let message = MessageEnvelope::new(
        "orders.created",
        vec![1, 2, 3],
        DeliverySemantics::new(DeliveryGuarantee::AtLeastOnce),
    );
    let command = ConnectionCommand::Subscribe {
        connection_id: connection_id.clone(),
        consumer: ConsumerSelection::Address("orders.#".to_owned()),
        delivery: DeliverySemantics::new(DeliveryGuarantee::AtLeastOnce),
    };

    assert_eq!(message.address, "orders.created");
    assert!(matches!(
        command,
        ConnectionCommand::Subscribe {
            connection_id: actual_connection_id,
            consumer: ConsumerSelection::Address(address),
            delivery: DeliverySemantics {
                guarantee: DeliveryGuarantee::AtLeastOnce,
                ..
            },
        } if actual_connection_id == connection_id && address == "orders.#"
    ));
}
