package org.correomqtt.business.utils;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.Setter;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashMap;
import java.util.Map;

@Getter
@Setter
@Builder
@AllArgsConstructor
public class CorreoMqttConnection {

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoMqttConnection.class);

    private int sort;
    private ConnectionConfigDTO configDTO;
    private CorreoMqttClient client;
    private Map<String, CorreoMqttClient> secondaryClients= new HashMap<>();

    public CorreoMqttConnection(ConnectionConfigDTO configDTO, int sort){
        this.sort = sort;
        this.configDTO = configDTO;
    }
}
