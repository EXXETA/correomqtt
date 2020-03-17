package com.exxeta.correomqtt.business.exception;

public class CorreoMqttNoRetriesLeftException extends CorreoMqttException {
    @Override
    public String getInfo() {
        return resources.getString("correoMqttNoRetriesLeftExceptionInfo");
    }
}
