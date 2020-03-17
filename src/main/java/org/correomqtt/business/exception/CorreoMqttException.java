package com.exxeta.correomqtt.business.exception;

import com.exxeta.correomqtt.business.services.ConfigService;

import java.util.ResourceBundle;

public abstract class CorreoMqttException extends RuntimeException {
    static ResourceBundle resources;

    CorreoMqttException(){
        resources = ResourceBundle.getBundle("com.exxeta.correomqtt.i18n", ConfigService.getInstance().getSettings().getCurrentLocale());
    }

    CorreoMqttException(String message){
        super(message);
    }

    CorreoMqttException(Throwable cause) {
        super(cause);
    }

    public abstract String getInfo();

}
