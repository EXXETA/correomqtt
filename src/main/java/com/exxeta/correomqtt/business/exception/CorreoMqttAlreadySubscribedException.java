package com.exxeta.correomqtt.business.exception;

import com.exxeta.correomqtt.business.model.SubscriptionDTO;

public class CorreoMqttAlreadySubscribedException extends CorreoMqttException {

    private final String connectionId;
    private final SubscriptionDTO subcriptionDTO;

    public CorreoMqttAlreadySubscribedException(String connectionId, SubscriptionDTO subscriptionDTO) {
        super();
        this.connectionId = connectionId;
        this.subcriptionDTO = subscriptionDTO;
    }

    @Override
    public String getInfo() {
        return subcriptionDTO.getTopic() + " " + resources.getString("correoMqttAlreadySubscribedExceptionInfo");
    }
}
