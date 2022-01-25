package org.correomqtt.business.exception;

import com.hivemq.client.mqtt.mqtt3.message.subscribe.suback.Mqtt3SubAckReturnCode;

import java.util.List;
import java.util.stream.Collectors;

public class CorreoMqtt3SubscriptionFailed extends CorreoMqttException {

    private final List<Mqtt3SubAckReturnCode> mqtt3ReturnCodes;

    public CorreoMqtt3SubscriptionFailed(List<Mqtt3SubAckReturnCode> returnCodes) {
        this.mqtt3ReturnCodes = returnCodes;
    }

    @Override
    public String getInfo() {
        return resources.getString("correoMqttSubscriptionFailedInfo") + " " + mqtt3ReturnCodes.stream()
                .map(c -> String.valueOf(c.getCode()))
                .collect(Collectors.joining(", "));
    }
}
