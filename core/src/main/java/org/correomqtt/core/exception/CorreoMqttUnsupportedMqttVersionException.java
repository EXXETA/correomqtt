package org.correomqtt.core.exception;

public class CorreoMqttUnsupportedMqttVersionException extends CorreoMqttException {
    @Override
    public String getInfo() {
        return "Unsupported MQTT version.";
    }
}
