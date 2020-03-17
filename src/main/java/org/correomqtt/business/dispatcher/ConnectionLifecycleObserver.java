package com.exxeta.correomqtt.business.dispatcher;

import java.util.concurrent.atomic.AtomicInteger;

public interface ConnectionLifecycleObserver extends BaseConnectionObserver {
    void onDisconnectFromConnectionDeleted(String connectionId);

    void onConnect();

    void onConnectRunning();

    void onConnectionFailed(Throwable message);

    default void onConnectionCanceled() {

    }

    void onConnectionLost();

    void onDisconnect();

    default void onConnectScheduled() {

    }

    default void onDisconnectCanceled() {

    }

    void onDisconnectFailed(Throwable exception);

    void onDisconnectRunning();

    default void onDisconnectScheduled() {

    }

    void onConnectionReconnected();

    void onReconnectFailed(AtomicInteger triedReconnects, int maxReconnects);
}
