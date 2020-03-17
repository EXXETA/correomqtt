package org.correomqtt.gui.model;

public enum ConnectionState {
    CONNECTED("connected"),
    CONNECTING("connecting"),
    DISCONNECTING("disconnecting"),
    DISCONNECTED_GRACEFUL("graceful"),
    DISCONNECTED_UNGRACEFUL("ungraceful");

    private final String cssClass;

    ConnectionState(String cssClass) {
        this.cssClass = cssClass;
    }

    public String getCssClass() {
        return cssClass;
    }
}
