package org.correomqtt.business.exception;

public class CorreoMqttUnsupportedMqttVersionException extends CorreoMqttException {
    @Override
    public String getInfo() {
        return resources.getString("correoMqttUnsupportedMqttVersionExceptionInfo");
    }
}
