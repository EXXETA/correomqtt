package org.correomqtt.business.exception;

import org.correomqtt.business.services.SettingsService;

import java.util.ResourceBundle;

public abstract class CorreoMqttException extends RuntimeException {
    static ResourceBundle resources;

    CorreoMqttException(){
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsService.getInstance().getSettings().getCurrentLocale());
    }

    CorreoMqttException(String message){
        super(message);
    }

    CorreoMqttException(Throwable cause) {
        super(cause);
    }

    public abstract String getInfo();

}
