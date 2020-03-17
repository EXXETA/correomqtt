package org.correomqtt.business.dispatcher;

import java.util.concurrent.atomic.AtomicInteger;

public class ConnectionLifecycleDispatcher extends BaseConnectionDispatcher<ConnectionLifecycleObserver> {

    private static ConnectionLifecycleDispatcher instance;

    private ConnectionLifecycleDispatcher() {
    }

    public static synchronized ConnectionLifecycleDispatcher getInstance() {
        if (instance == null) {
            instance = new ConnectionLifecycleDispatcher();
        }
        return instance;
    }

    public void onConnect(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onConnect);
    }

    public void onConnectRunning(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onConnectRunning);
    }

    public void onConnectionFailed(String connectionId, Throwable exception) {
        triggerFiltered(connectionId, o -> o.onConnectionFailed(exception));
    }

    public void onConnectionLost(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onConnectionLost);
    }

    public void onConnectionReconnected(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onConnectionReconnected);
    }

    public void onConnectionCanceled(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onConnectionCanceled);
    }

    public void onConnectScheduled(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onConnectScheduled);
    }

    public void onDisconnect(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onDisconnect);
        trigger(o -> o.onDisconnectFromConnectionDeleted(connectionId));
    }

    public void onDisconnectRunning(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onDisconnectRunning);
    }

    public void onDisconnectFailed(String connectionId, Throwable exception) {
        triggerFiltered(connectionId, (o -> o.onDisconnectFailed(exception)));
    }

    public void onDisconnectCanceled(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onDisconnectCanceled);
    }

    public void onDisconnectScheduled(String connectionId) {
        triggerFiltered(connectionId, ConnectionLifecycleObserver::onDisconnectScheduled);
    }

    public void onReconnectFailed(String connectionId, AtomicInteger triedReconnects, int maxReconnects) {
        triggerFiltered(connectionId,o -> {
            o.onReconnectFailed(triedReconnects, maxReconnects);
        });
    }
}
