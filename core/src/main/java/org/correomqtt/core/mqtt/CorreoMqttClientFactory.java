package org.correomqtt.core.mqtt;

import org.correomqtt.core.exception.CorreoMqttUnsupportedMqttVersionException;
import org.correomqtt.core.model.ConnectionConfigDTO;

import javax.inject.Inject;

public class CorreoMqttClientFactory {

    private final CorreoMqtt3Client.Factory correoMqtt3ClientFactory;
    private final CorreoMqtt5Client.Factory correoMqtt5ClientFactory;

    @Inject
    public CorreoMqttClientFactory(CorreoMqtt3Client.Factory correoMqtt3ClientFactory,
                                    CorreoMqtt5Client.Factory correoMqtt5ClientFactory) {
        // private constructor
        this.correoMqtt3ClientFactory = correoMqtt3ClientFactory;
        this.correoMqtt5ClientFactory = correoMqtt5ClientFactory;
    }

    public CorreoMqttClient createClient(ConnectionConfigDTO configDTO) {
        return switch (configDTO.getMqttVersion()) {
            case MQTT_3_1_1 -> correoMqtt3ClientFactory.create(configDTO);
            case MQTT_5_0 -> correoMqtt5ClientFactory.create(configDTO);
            default -> throw new CorreoMqttUnsupportedMqttVersionException();
        };
    }

}
