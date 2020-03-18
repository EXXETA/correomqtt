package org.correomqtt.business.exception;

public abstract class CorreoMqttSubscriptionFailed extends CorreoMqttException {

    @Override
    public String getInfo() {
        return resources.getString("correoMqttSubscriptionFailedInfo");
    }
}
