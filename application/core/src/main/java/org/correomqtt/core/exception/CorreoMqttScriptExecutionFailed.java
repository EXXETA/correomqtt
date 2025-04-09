package org.correomqtt.core.exception;

import java.io.IOException;

public class CorreoMqttScriptExecutionFailed extends CorreoMqttException {

    public CorreoMqttScriptExecutionFailed(IOException e) {
        super(e);
    }

    @Override
    public String getInfo() {
        return this.getCause().getMessage();
    }
}
