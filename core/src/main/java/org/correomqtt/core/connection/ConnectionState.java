package org.correomqtt.core.connection;

public enum ConnectionState {
    CONNECTED,
    CONNECTING,
    RECONNECTING,
    DISCONNECTING,
    DISCONNECTED_GRACEFUL,
    DISCONNECTED_UNGRACEFUL;
}
