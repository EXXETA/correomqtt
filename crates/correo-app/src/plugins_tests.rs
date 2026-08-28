fn joined_ids(ids: &[String]) -> String {
    if ids.is_empty() {
        "none".to_owned()
    } else {
        ids.join(", ")
    }
}

fn joined_row_ids(rows: &[PluginMarketplaceRow]) -> String {
    if rows.is_empty() {
        "none".to_owned()
    } else {
        rows.iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use correo_plugins::{DetailByteTransformResponse, HostCapabilityGrants, SavePayloadActionDto};

    fn detail_transform_with_save() -> HookOutput {
        HookOutput::DetailByteTransform(DetailByteTransformResponse {
            abi_version: correo_plugins::ABI_VERSION,
            bytes: b"transformed".to_vec(),
            content_type: Some("application/octet-stream".to_owned()),
            host_actions: vec![HostActionDto::SavePayload(SavePayloadActionDto {
                suggested_file_name: "transformed.bin".to_owned(),
                bytes: b"saved".to_vec(),
                content_type: Some("application/octet-stream".to_owned()),
            })],
        })
    }

    #[test]
    fn detail_transform_maps_granted_save_payload() {
        let execution = hook_execution(
            detail_transform_with_save(),
            &HostCapabilityGrants {
                message_save: true,
                ..Default::default()
            },
            None,
        )
        .unwrap();

        assert_eq!(
            execution.host_actions,
            vec![PluginHostAction::SavePayload(PluginSavePayload {
                suggested_file_name: "transformed.bin".to_owned(),
                bytes: b"saved".to_vec(),
                content_type: Some("application/octet-stream".to_owned()),
            })]
        );
        assert_eq!(
            execution.output,
            PluginHookOutput::DetailBytes(DetailBytesOutput {
                bytes: b"transformed".to_vec(),
                content_type: Some("application/octet-stream".to_owned()),
            })
        );
    }

    #[test]
    fn detail_transform_rejects_ungranted_save_payload() {
        let error = hook_execution(
            detail_transform_with_save(),
            &HostCapabilityGrants::default(),
            None,
        )
        .unwrap_err();
        assert_eq!(
            error.message,
            "plugin is not granted the message-save host capability"
        );
    }

    #[test]
    fn v2_transport_transform_maps_transport_input() {
        let mut message = MessageEnvelope::new(
            "orders.created",
            vec![1, 2, 3],
            DeliverySemantics::new(DeliveryGuarantee::AtLeastOnce),
        );
        message.metadata.insert(
            NamespacedName::new("mqtt.retained").unwrap(),
            serde_json::Value::Bool(true),
        );
        let call = PluginHookCall {
            plugin_id: "org.correomqtt.plugins.v2".to_owned(),
            hook: PluginHookKind::OutgoingTransform,
            target: "orders.created".to_owned(),
            config: serde_json::Value::Null,
            input: PluginHookInput::TransportMessage(PluginTransportMessage {
                message,
                consumer: None,
                capabilities: TransportCapabilities::default(),
            }),
        };

        let HookInvocation::OutgoingTransportMessageTransform(request) =
            hook_invocation(call, PluginAbi::V2).unwrap()
        else {
            panic!("expected a V2 transport invocation");
        };
        assert_eq!(request.input.message.address, "orders.created");
        assert_eq!(request.input.message.body, vec![1, 2, 3]);
        assert_eq!(
            request.input.message.delivery,
            DeliveryGuaranteeDto::AtLeastOnce
        );
        assert_eq!(
            request
                .input
                .message
                .metadata
                .get(&NamespacedNameDto::new("mqtt.retained").unwrap()),
            Some(&serde_json::Value::Bool(true))
        );
    }

    #[test]
    fn v1_transport_bridge_preserves_transport_context() {
        let mut message = MessageEnvelope::new(
            "orders.created",
            vec![1, 2, 3],
            DeliverySemantics::new(DeliveryGuarantee::AtLeastOnce),
        );
        message.metadata.insert(
            NamespacedName::new("mqtt.retained").unwrap(),
            serde_json::Value::Bool(false),
        );
        message.metadata.insert(
            NamespacedName::new("mqtt.packet_id").unwrap(),
            serde_json::Value::from(42),
        );
        let mut capabilities = TransportCapabilities::default();
        capabilities.insert(TransportCapability::new("mqtt.retained").unwrap());
        let input = PluginTransportMessage {
            message,
            consumer: Some(ConsumerSelection::Address("orders.created".to_owned())),
            capabilities,
        };

        let output = v1_transport_output(
            MessageTransform::Replace(PluginMessage {
                topic: "orders.updated".to_owned(),
                payload: vec![4, 5, 6],
                qos: QosLevel::Two,
                retained: true,
            }),
            Some(input),
        )
        .unwrap();

        let PluginHookOutput::TransportMessageTransform(TransportMessageTransform::Replace(output)) =
            output
        else {
            panic!("expected a transport replacement");
        };
        assert_eq!(output.message.address, "orders.updated");
        assert_eq!(output.message.body, vec![4, 5, 6]);
        assert_eq!(
            output.message.delivery,
            DeliverySemantics::new(DeliveryGuarantee::ExactlyOnce)
        );
        assert_eq!(
            output.message.metadata.get_value("mqtt.packet_id"),
            Some(&serde_json::Value::from(42))
        );
        assert_eq!(
            output.message.metadata.get_value("mqtt.retained"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            output.consumer,
            Some(ConsumerSelection::Address("orders.created".to_owned()))
        );
        assert!(output
            .capabilities
            .iter()
            .any(|capability| capability.as_str() == "mqtt.retained"));
    }
}
