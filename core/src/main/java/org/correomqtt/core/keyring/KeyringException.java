package org.correomqtt.core.keyring;

import org.correomqtt.core.exception.CorreoMqttException;

public class KeyringException extends CorreoMqttException {

    private final Exception exception;
    private final String msg;

    public KeyringException(String msg){
        super();
        this.msg = msg;
        this.exception = null;
    }

    public KeyringException(String msg, Exception e){
        super();
        this.msg = msg;
        this.exception = e;
    }

    @Override
    public String getInfo() {
        return msg + ((exception == null) ? "": " " + exception.getMessage());
    }
}
