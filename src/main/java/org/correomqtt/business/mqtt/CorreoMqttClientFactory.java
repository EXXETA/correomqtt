package org.correomqtt.business.mqtt;

import org.correomqtt.business.exception.CorreoMqttUnsupportedMqttVersionException;
import org.correomqtt.business.model.ConnectionConfigDTO;

public class CorreoMqttClientFactory {

    private CorreoMqttClientFactory(){
        // private constructor
    }

    public static CorreoMqttClient createClient(ConnectionConfigDTO configDTO){
        switch(configDTO.getMqttVersion()){
            case MQTT_3_1_1:
                return new CorreoMqtt3Client(configDTO);
            case MQTT_5_0:
                return new CorreoMqtt5Client(configDTO);
            default:
                throw new CorreoMqttUnsupportedMqttVersionException();
        }
    }

}
