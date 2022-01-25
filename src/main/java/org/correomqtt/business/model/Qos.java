package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import com.hivemq.client.mqtt.datatypes.MqttQos;

import java.util.Arrays;

public enum Qos {
    AT_MOST_ONCE(MqttQos.AT_MOST_ONCE, "qosEnumAtMostOnce"),
    AT_LEAST_ONCE(MqttQos.AT_LEAST_ONCE, "qosEnumAtLeastOnce"),
    EXACTLY_ONCE(MqttQos.EXACTLY_ONCE, "qosEnumExactlyOnce");

    private final MqttQos mqttQos;
    private final String description;

    Qos(MqttQos mqttQos, String description) {
        this.mqttQos = mqttQos;
        this.description = description;
    }

    public static Qos valueOf(MqttQos mqttQos) {
        return Arrays.stream(values())
                     .filter(v -> mqttQos == v.getMqttQos())
                     .findFirst()
                     .orElseThrow(() -> new IllegalArgumentException("Qos can not be matched."));
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static Qos fromJsonValue(int value) {
        if (value < 0 || value > 2) {
            return null;
        }
        return values()[value];
    }

    @Override
    public String toString() {
        return "QoS " + mqttQos.ordinal();
    }

    public MqttQos getMqttQos() {
        return mqttQos;
    }

    @SuppressWarnings("unused")
    @JsonValue
    public int toJsonValue() {
        return ordinal();
    }

    public String getDescription() {
        return description;
    }
}
