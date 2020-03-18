package org.correomqtt.business.exception;

import org.correomqtt.business.services.ConfigService;

import java.util.ResourceBundle;

public abstract class CorreoMqttException extends RuntimeException {
    static ResourceBundle resources;

    CorreoMqttException(){
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());
    }

    CorreoMqttException(String message){
        super(message);
    }

    CorreoMqttException(Throwable cause) {
        super(cause);
    }

    public abstract String getInfo();

}
