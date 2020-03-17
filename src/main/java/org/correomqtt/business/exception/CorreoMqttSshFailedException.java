package com.exxeta.correomqtt.business.exception;

public class CorreoMqttSshFailedException extends CorreoMqttException {

    public CorreoMqttSshFailedException(Exception e) {
        super(e);
    }

    @Override
    public String getInfo() {
        return resources.getString("correoMqttSshFailedExceptionInfo") + ": " + getCause().getMessage();
    }
}
