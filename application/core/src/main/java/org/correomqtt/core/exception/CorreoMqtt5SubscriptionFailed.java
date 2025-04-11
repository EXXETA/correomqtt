package org.correomqtt.core.exception;

import com.hivemq.client.mqtt.mqtt5.message.subscribe.suback.Mqtt5SubAckReasonCode;

import java.util.List;
import java.util.stream.Collectors;

public class CorreoMqtt5SubscriptionFailed extends CorreoMqttException {

    private final List<Mqtt5SubAckReasonCode> mqtt5ReasonCodes;

    public CorreoMqtt5SubscriptionFailed(List<Mqtt5SubAckReasonCode> returnCodes) {
        this.mqtt5ReasonCodes = returnCodes;
    }

    @Override
    public String getInfo() {
        return "Subscription failed. " + mqtt5ReasonCodes.stream()
                .map(c -> String.valueOf(c.getCode()))
                .collect(Collectors.joining(", "));
    }
}
