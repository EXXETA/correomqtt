package com.exxeta.correomqtt.business.exception;

import com.hivemq.client.mqtt.mqtt3.message.subscribe.suback.Mqtt3SubAckReturnCode;

import java.util.List;

public class CorreoMqtt3SubscriptionFailed extends CorreoMqttSubscriptionFailed {

    private final List<Mqtt3SubAckReturnCode> mqtt3ReturnCodes;

    public CorreoMqtt3SubscriptionFailed(List<Mqtt3SubAckReturnCode> returnCodes) {
        this.mqtt3ReturnCodes = returnCodes;
    }
}
