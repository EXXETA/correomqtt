package com.exxeta.correomqtt.business.services;

import com.exxeta.correomqtt.business.dispatcher.ConnectionLifecycleDispatcher;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class DisconnectService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(DisconnectService.class);

    public DisconnectService(String connectionId) {
        super(connectionId);
    }

    public void disconnect() {
        LOGGER.info(getConnectionMarker(), "Start disconnecting.");
        callSafeOnClient(c -> c.disconnect(true));
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Disconnected.");
        getConnection().setClient(null);
        ConnectionLifecycleDispatcher.getInstance().onDisconnect(connectionId);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Disconnect cancelled");
        ConnectionLifecycleDispatcher.getInstance().onDisconnectCanceled(connectionId);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.warn(getConnectionMarker(), "Disconnecting from broker failed.", exception);
        ConnectionLifecycleDispatcher.getInstance().onDisconnectFailed(connectionId, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.debug(getConnectionMarker(), "Disconnect running.");
        ConnectionLifecycleDispatcher.getInstance().onDisconnectRunning(connectionId);
    }

    @Override
    public void onScheduled() {
        LOGGER.debug(getConnectionMarker(), "Disconnect scheduled.");
        ConnectionLifecycleDispatcher.getInstance().onDisconnectScheduled(connectionId);
    }
}
