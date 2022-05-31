package org.correomqtt.business.exception;

public class CorreoMqttSshFailedException extends CorreoMqttException {

    public CorreoMqttSshFailedException(Exception e) {
        super(e);
    }

    public CorreoMqttSshFailedException(String message, Exception e) {
        super(message, e);
    }

    @Override
    public String getInfo() {
        return resources.getString("correoMqttSshFailedExceptionInfo") + ": " + getCause().getMessage();
    }
}
