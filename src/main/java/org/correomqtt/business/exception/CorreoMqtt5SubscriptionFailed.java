package com.exxeta.correomqtt.business.exception;

import com.hivemq.client.mqtt.mqtt5.message.subscribe.suback.Mqtt5SubAckReasonCode;

import java.util.List;

public class CorreoMqtt5SubscriptionFailed extends CorreoMqttSubscriptionFailed {

    private final List<Mqtt5SubAckReasonCode> mqtt5ReasonCodes;

    public CorreoMqtt5SubscriptionFailed(List<Mqtt5SubAckReasonCode> returnCodes) {
        this.mqtt5ReasonCodes = returnCodes;
    }
}
