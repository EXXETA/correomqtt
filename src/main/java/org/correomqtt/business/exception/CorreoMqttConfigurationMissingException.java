package org.correomqtt.business.exception;

public class CorreoMqttConfigurationMissingException extends CorreoMqttException{

    @Override
    public String getInfo(){
        return resources.getString("CorreoMqttConfigurationMissingException");
    }
}
