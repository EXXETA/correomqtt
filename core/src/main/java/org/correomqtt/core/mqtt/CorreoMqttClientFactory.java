package org.correomqtt.core.mqtt;

import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.exception.CorreoMqttUnsupportedMqttVersionException;
import org.correomqtt.core.model.ConnectionConfigDTO;

@DefaultBean
public class CorreoMqttClientFactory {

    private final CorreoMqtt3ClientFactory correoMqtt3ClientFactory;
    private final CorreoMqtt5ClientFactory correoMqtt5ClientFactory;

    @Inject
    public CorreoMqttClientFactory(CorreoMqtt3ClientFactory correoMqtt3ClientFactory,
                                   CorreoMqtt5ClientFactory correoMqtt5ClientFactory) {
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
