package org.correomqtt.core.exception;

public abstract class CorreoMqttException extends RuntimeException {

    protected CorreoMqttException() {
    }

    protected CorreoMqttException(String message) {
        super(message);
    }

    protected CorreoMqttException(String message, Throwable cause) {
        super(message, cause);
    }

    protected CorreoMqttException(Throwable cause) {
        super(cause);
    }


    public abstract String getInfo();

}
