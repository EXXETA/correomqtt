use correo_mqtt::{IncomingMessage, PublishRequest, Qos, TopicName};
use serde_json::json;

use crate::mqtt::{message_from_incoming, message_from_publish};
use crate::{
    DeliveryGuarantee, DeliverySemantics, FormattedMessageDetail, MessageDetailFormat,
    MessageDiagnosticRow, NamespacedName, PluginDiagnosticSeverity, PluginHookKind, PluginMessage,
    PluginTransportMessage, QosLevel, TransportCapabilities, TransportCapability,
};

#[derive(Debug, Clone)]
pub(super) struct ActiveHook {
    pub plugin_id: String,
    pub hook: PluginHookKind,
    pub target: String,
    pub config_json: String,
}

pub(super) fn plugin_message_from_publish(request: &PublishRequest) -> PluginMessage {
    PluginMessage {
        topic: request.topic.as_str().to_owned(),
        payload: request.payload.clone(),
        qos: qos_level(request.qos),
        retained: request.retain,
    }
}

pub(super) fn plugin_message_from_incoming(message: &IncomingMessage) -> PluginMessage {
    PluginMessage {
        topic: message.topic.as_str().to_owned(),
        payload: message.payload.clone(),
        qos: qos_level(message.qos),
        retained: message.retain,
    }
}

pub(super) fn incoming_from_plugin_message(
    mut original: IncomingMessage,
    message: PluginMessage,
) -> Result<IncomingMessage, String> {
    original.topic = TopicName::new(message.topic).map_err(|error| error.to_report().message)?;
    original.payload = message.payload;
    original.qos = mqtt_qos(message.qos);
    original.retain = message.retained;
    Ok(original)
}

pub(super) fn plugin_transport_message_from_publish(
    request: &PublishRequest,
    message: PluginMessage,
) -> PluginTransportMessage {
    merge_plugin_message(message_from_publish(request), message)
}

pub(super) fn plugin_transport_message_from_incoming(
    incoming: &IncomingMessage,
    message: PluginMessage,
) -> PluginTransportMessage {
    merge_plugin_message(message_from_incoming(incoming), message)
}

pub(super) fn plugin_message_from_transport(
    input: PluginTransportMessage,
    retained_when_absent: bool,
    capabilities: &TransportCapabilities,
) -> Result<PluginMessage, String> {
    let retained_capability = TransportCapability::new("mqtt.retained")
        .expect("MQTT transport capabilities are namespaced");
    let retained = match input.message.metadata.get_value("mqtt.retained") {
        Some(serde_json::Value::Bool(value))
            if capabilities
                .iter()
                .any(|capability| capability == &retained_capability) =>
        {
            *value
        }
        Some(serde_json::Value::Bool(_)) => {
            return Err(
                "transport metadata mqtt.retained is not mutable for this connection".to_owned(),
            )
        }
        Some(_) => return Err("transport metadata mqtt.retained must be a boolean".to_owned()),
        None => retained_when_absent,
    };
    Ok(PluginMessage {
        topic: input.message.address,
        payload: input.message.body,
        qos: match input.message.delivery.guarantee {
            DeliveryGuarantee::AtMostOnce => QosLevel::Zero,
            DeliveryGuarantee::AtLeastOnce => QosLevel::One,
            DeliveryGuarantee::ExactlyOnce => QosLevel::Two,
        },
        retained,
    })
}

fn merge_plugin_message(
    mut message: crate::MessageEnvelope,
    plugin: PluginMessage,
) -> PluginTransportMessage {
    message.address = plugin.topic;
    message.body = plugin.payload;
    message.delivery = DeliverySemantics::new(match plugin.qos {
        QosLevel::Zero => DeliveryGuarantee::AtMostOnce,
        QosLevel::One => DeliveryGuarantee::AtLeastOnce,
        QosLevel::Two => DeliveryGuarantee::ExactlyOnce,
    });
    message.metadata.insert(
        mqtt_metadata_key("mqtt.qos"),
        json!(match plugin.qos {
            QosLevel::Zero => "at_most_once",
            QosLevel::One => "at_least_once",
            QosLevel::Two => "exactly_once",
        }),
    );
    message
        .metadata
        .insert(mqtt_metadata_key("mqtt.retained"), json!(plugin.retained));
    let mut capabilities = TransportCapabilities::default();
    capabilities.insert(
        TransportCapability::new("mqtt.retained")
            .expect("MQTT transport capabilities are namespaced"),
    );
    PluginTransportMessage {
        message,
        consumer: None,
        capabilities,
    }
}

fn mqtt_metadata_key(value: &'static str) -> NamespacedName {
    NamespacedName::new(value).expect("MQTT metadata keys are namespaced")
}

pub(super) fn message_diagnostic(
    hook: &ActiveHook,
    severity: PluginDiagnosticSeverity,
    message: &str,
) -> MessageDiagnosticRow {
    MessageDiagnosticRow {
        severity,
        hook: Some(hook.hook),
        plugin_id: Some(hook.plugin_id.clone()),
        message: message.to_owned(),
    }
}

pub(super) fn plain_detail(
    bytes: Vec<u8>,
    content_type: Option<String>,
    diagnostics: Vec<MessageDiagnosticRow>,
) -> FormattedMessageDetail {
    FormattedMessageDetail {
        format: MessageDetailFormat::PlainText,
        text: String::from_utf8_lossy(&bytes).into_owned(),
        content_type,
        diagnostics,
    }
}

pub(super) fn qos_level(qos: Qos) -> QosLevel {
    match qos {
        Qos::AtMostOnce => QosLevel::Zero,
        Qos::AtLeastOnce => QosLevel::One,
        Qos::ExactlyOnce => QosLevel::Two,
    }
}

pub(super) fn mqtt_qos(qos: QosLevel) -> Qos {
    match qos {
        QosLevel::Zero => Qos::AtMostOnce,
        QosLevel::One => Qos::AtLeastOnce,
        QosLevel::Two => Qos::ExactlyOnce,
    }
}

pub(super) fn topic_matches_filter(topic: &str, filter: &str) -> bool {
    let filter = filter.trim();
    if filter.is_empty() || filter == "#" {
        return true;
    }
    let topic_levels = topic.split('/').collect::<Vec<_>>();
    let filter_levels = filter.split('/').collect::<Vec<_>>();

    for (index, filter_level) in filter_levels.iter().enumerate() {
        match *filter_level {
            "#" => return index == filter_levels.len() - 1,
            "+" => {
                if topic_levels.get(index).is_none() {
                    return false;
                }
            }
            literal if topic_levels.get(index) != Some(&literal) => return false,
            _ => {}
        }
    }

    topic_levels.len() == filter_levels.len()
}
