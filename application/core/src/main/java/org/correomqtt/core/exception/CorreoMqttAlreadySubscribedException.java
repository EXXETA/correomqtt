package org.correomqtt.core.exception;

import org.correomqtt.core.model.SubscriptionDTO;

public class CorreoMqttAlreadySubscribedException extends CorreoMqttException {

    private final String connectionId;
    private final transient SubscriptionDTO subcriptionDTO;

    public CorreoMqttAlreadySubscribedException(String connectionId, SubscriptionDTO subscriptionDTO) {
        super();
        this.connectionId = connectionId;
        this.subcriptionDTO = subscriptionDTO;
    }

    @Override
    public String getInfo() {
        return "[" + connectionId + "] " + subcriptionDTO.getTopic() + " already subscribed.";
    }
}
