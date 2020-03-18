package org.correomqtt.business.exception;

import java.text.MessageFormat;

public class CorreoMqttDisconnectException extends CorreoMqttException {

    public CorreoMqttDisconnectException(String message) {
        super(message);
    }

    public CorreoMqttDisconnectException(String message, Object... parameter) {
        super(MessageFormat.format(message, parameter));
    }

    @Override
    public String getInfo() {
        return resources.getString("correoMqttDisconnectExceptionInfo") + ": " + getMessage();
    }
}
