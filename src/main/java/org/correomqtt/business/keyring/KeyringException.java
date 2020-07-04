package org.correomqtt.business.keyring;

import org.correomqtt.business.exception.CorreoMqttException;

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
