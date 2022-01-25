package org.correomqtt.plugin.exception;

import org.correomqtt.business.exception.CorreoMqttException;

import java.util.Objects;

public class CorreoMqttPluginUpdateException extends RuntimeException{

    public CorreoMqttPluginUpdateException(String e) { super(e); }

    public String getInfo() {
        Throwable cause1 = getCause();
        if (cause1 != null) {
            Throwable cause2 = cause1.getCause();
            return getSafeMessage(Objects.requireNonNullElse(cause2, cause1));
        } else {
            return cause1.getMessage();
        }
    }

    private String getSafeMessage(Throwable cause){
        if (cause instanceof CorreoMqttException) {
            return ((CorreoMqttException) cause).getInfo();
        } else {
            String msg = cause.getMessage();
            if(msg == null){
                msg = cause.getClass().getSimpleName();
            }
            return msg;
        }
    }
}
