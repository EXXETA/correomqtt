package com.exxeta.correomqtt.business.exception;

import com.hivemq.client.mqtt.mqtt3.message.connect.connack.Mqtt3ConnAckReturnCode;
import com.hivemq.client.mqtt.mqtt5.message.connect.connack.Mqtt5ConnAckReasonCode;

public class CorreoMqttConnectionFailedException extends CorreoMqttException {

    private final Mqtt3ConnAckReturnCode mqtt3ReturnCode;
    private final Mqtt5ConnAckReasonCode mqtt5ReasonCode;

    public CorreoMqttConnectionFailedException(Mqtt3ConnAckReturnCode returnCode) {
        this.mqtt3ReturnCode = returnCode;
        this.mqtt5ReasonCode = null;
    }

    public CorreoMqttConnectionFailedException(Mqtt5ConnAckReasonCode reasonCode) {
        this.mqtt3ReturnCode = null;
        this.mqtt5ReasonCode = reasonCode;
    }

    @Override
    public String getInfo() {
        if (mqtt3ReturnCode != null) {
            return resources.getString("correoMqttConnectionFailedExceptionMqtt3ReturnCode") + ": " + mqtt3ReturnCode.name();
        } else if (mqtt5ReasonCode != null) {
            return resources.getString("correoMqttConnectionFailedExceptionMqtt5ReasonCode") + ": " + mqtt5ReasonCode.name();
        }

        throw new IllegalStateException("No valid code found.");
    }
}
