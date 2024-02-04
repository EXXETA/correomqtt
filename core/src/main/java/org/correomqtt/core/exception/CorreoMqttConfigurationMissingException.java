package org.correomqtt.core.exception;

public class CorreoMqttConfigurationMissingException extends CorreoMqttException{

    @Override
    public String getInfo(){
        return "Configuration missing";
    }
}
