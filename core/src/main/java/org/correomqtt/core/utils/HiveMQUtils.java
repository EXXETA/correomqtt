package org.correomqtt.core.utils;

import com.hivemq.client.mqtt.datatypes.MqttQos;

public class HiveMQUtils {

    private HiveMQUtils() {
        // private constructor
    }

    public static MqttQos qosToEnum(int qos) {
        switch (qos) {
            case 0:
                return MqttQos.AT_MOST_ONCE;
            case 1:
                return MqttQos.AT_LEAST_ONCE;
            case 2:
                return MqttQos.EXACTLY_ONCE;
            default:
                throw new IllegalArgumentException("Invalid QOS");
        }
    }
}
