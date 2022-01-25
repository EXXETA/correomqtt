package org.correomqtt.business.exception;

import org.correomqtt.business.provider.SettingsProvider;

import java.util.ResourceBundle;

public abstract class CorreoMqttException extends RuntimeException {
    static ResourceBundle resources;

    protected CorreoMqttException(){
        resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());
    }

    protected CorreoMqttException(String message){
        super(message);
    }

    protected CorreoMqttException(Throwable cause) {
        super(cause);
    }

    public abstract String getInfo();

}
