package org.correomqtt.core.exception;

public class CorreoMqttSshFailedException extends CorreoMqttException {

    public CorreoMqttSshFailedException(Exception e) {
        super(e);
    }

    public CorreoMqttSshFailedException(String message, Exception e) {
        super(message, e);
    }

    @Override
    public String getInfo() {
        return "SSH failed: " + getCause().getMessage();
    }
}
