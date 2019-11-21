package com.exxeta.correomqtt.business.exception;

import java.util.Objects;

public class CorreoMqttExecutionException extends CorreoMqttException {
    public CorreoMqttExecutionException(Exception e) {
        super(e);
    }

    @Override
    public String getInfo() {
        Throwable cause1 = getCause();
        if (cause1 != null) {
            Throwable cause2 = cause1.getCause();
            return getSafeMessage(Objects.requireNonNullElse(cause2, cause1));
        } else {
            return resources.getString("correoMqttExecutionException");
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
