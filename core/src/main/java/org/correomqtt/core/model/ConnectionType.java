package org.correomqtt.core.model;

public enum ConnectionType {
    MQTT(true),
    KAFKA(false);

    private final boolean hasQos;
    ConnectionType(boolean hasQos){
        this.hasQos = hasQos;
    }

    public boolean hasQos(){
        return this.hasQos;
    }
}
