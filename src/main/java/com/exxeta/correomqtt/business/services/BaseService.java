package com.exxeta.correomqtt.business.services;

import com.exxeta.correomqtt.business.exception.CorreoMqttDisconnectException;
import com.exxeta.correomqtt.business.exception.CorreoMqttExecutionException;
import com.exxeta.correomqtt.business.model.ConnectionConfigDTO;
import com.exxeta.correomqtt.business.mqtt.CorreoMqttClient;
import com.exxeta.correomqtt.business.utils.ConnectionHolder;
import com.exxeta.correomqtt.business.utils.CorreoMqttConnection;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

abstract class BaseService implements BusinessService {

    protected final String connectionId;

    BaseService(String connectionId) {
        this.connectionId = connectionId;
    }

    CorreoMqttConnection getConnection() {
        return ConnectionHolder.getInstance().getConnection(connectionId);
    }

    CorreoMqttClient getClient() {
        return ConnectionHolder.getInstance().getClient(connectionId);
    }

    Marker getConnectionMarker() {
        ConnectionConfigDTO connectionConfig = ConnectionHolder.getInstance().getConfig(connectionId);
        if (connectionConfig == null) {
            return MarkerFactory.getMarker("Unknown");
        }
        return MarkerFactory.getMarker(connectionConfig.getName());
    }

    void callSafeOnClient(ClientConsumerThatMightFail consumer) {
        CorreoMqttClient client = getClient();
        if (client != null) {
            try {
                consumer.accept(client);
            } catch (Exception e) {
                throw new CorreoMqttExecutionException(e);
            }
        } else {
            throw new CorreoMqttDisconnectException("No client available.");
        }
    }

    @FunctionalInterface
    public interface ClientConsumerThatMightFail {
        void accept(CorreoMqttClient client) throws Exception;
    }
}
