package org.correomqtt.business.services;

import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import com.exxeta.correomqtt.business.mqtt.CorreoMqttClient;
import com.exxeta.correomqtt.business.mqtt.CorreoMqttClientFactory;
import com.exxeta.correomqtt.business.utils.ConnectionHolder;
import com.exxeta.correomqtt.business.utils.CorreoMqttConnection;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class ConnectService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectService.class);

    public ConnectService(String connectionId) {
        super(connectionId);
    }

    public void connect() {

        CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(connectionId);
        connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));

        callSafeOnClient(CorreoMqttClient::connect);
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Connecting to broker successfully.");
        ConnectionLifecycleDispatcher.getInstance().onConnect(connectionId);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Connecting to broker cancelled.");
        ConnectionLifecycleDispatcher.getInstance().onConnectionCanceled(connectionId);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info(getConnectionMarker(), "Connecting to broker failed: ", exception);
        ConnectionLifecycleDispatcher.getInstance().onConnectionFailed(connectionId, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.debug(getConnectionMarker(), "Connecting to broker running.");
        ConnectionLifecycleDispatcher.getInstance().onConnectRunning(connectionId);
    }

    @Override
    public void onScheduled() {
        LOGGER.debug(getConnectionMarker(), "Connecting to broker scheduled.");
        ConnectionLifecycleDispatcher.getInstance().onConnectScheduled(connectionId);
    }

}
