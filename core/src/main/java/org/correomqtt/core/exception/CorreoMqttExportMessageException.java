package org.correomqtt.core.exception;

import java.io.IOException;

public class CorreoMqttExportMessageException extends CorreoMqttException {

    public CorreoMqttExportMessageException(IOException e) {
        super(e);
    }

    @Override
    public String getInfo() {
        return getCause().getMessage();
    }
}
