package com.exxeta.correomqtt.business.exception;

public class CorreoMqttNotConnectedException extends CorreoMqttException {
    @Override
    public String getInfo() {
        return resources.getString("correoMqttNotConnectedExceptionInfo");
    }
}
