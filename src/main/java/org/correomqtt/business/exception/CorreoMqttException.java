package org.correomqtt.business.exception;

import org.correomqtt.business.provider.SettingsProvider;

import java.util.ResourceBundle;

public abstract class CorreoMqttException extends RuntimeException {

    final transient ResourceBundle resources;

    protected CorreoMqttException() {
        resources = initResources();
    }

    private ResourceBundle initResources() {
        return ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());
    }

    protected CorreoMqttException(String message) {
        super(message);
        resources = initResources();
    }

    protected CorreoMqttException(String message, Throwable cause) {
        super(message, cause);
        resources = initResources();
    }

    protected CorreoMqttException(Throwable cause) {
        super(cause);
        resources = initResources();
    }


    public abstract String getInfo();

}
