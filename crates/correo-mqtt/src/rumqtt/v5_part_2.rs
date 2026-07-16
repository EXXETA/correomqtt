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
