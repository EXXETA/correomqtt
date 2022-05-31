package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientFactory;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;
import org.correomqtt.gui.business.TaskFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.net.ssl.SSLException;
import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

public class ConnectService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ConnectService.class);

    public ConnectService(String connectionId) {
        super(connectionId);
    }

    public void connect() {

        CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(connectionId);
        connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));

        callSafeOnClient(this::connect);
    }

    private void connect(CorreoMqttClient correoMqttClient) {
        try {
            correoMqttClient.connect();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new CorreoMqttExecutionException(e);
        } catch (ExecutionException | TimeoutException | SSLException e) {
            throw new CorreoMqttExecutionException(e);
        }
    }

    public void reconnect() {
        CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(connectionId);
        Set<SubscriptionDTO> existingSubscriptions = connection.getClient().getSubscriptions();

        connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));
        callSafeOnClient(this::connect);

        existingSubscriptions.forEach(subscriptionDTO -> {
            SubscribeService subscribeService = new SubscribeService(connectionId, subscriptionDTO);
            subscribeService.subscribe();
        });
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
