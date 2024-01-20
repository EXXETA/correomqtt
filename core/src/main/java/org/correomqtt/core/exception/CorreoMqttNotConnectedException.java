package org.correomqtt.core.exception;

public class CorreoMqttNotConnectedException extends CorreoMqttException {
    @Override
    public String getInfo() {
        return "Not Connected.";
    }
}
