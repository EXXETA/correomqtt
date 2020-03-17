package com.exxeta.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import com.hivemq.client.mqtt.datatypes.MqttQos;

import java.util.Arrays;

public enum Qos {
    AT_MOST_ONCE(MqttQos.AT_MOST_ONCE, "qosEnumAtMostOnce"),
    AT_LEAST_ONCE(MqttQos.AT_LEAST_ONCE, "qosEnumAtLeastOnce"),
    EXACTLY_ONCE(MqttQos.EXACTLY_ONCE, "qosEnumExactlyOnce");

    private final MqttQos qos;
    private final String description;

    Qos(MqttQos qos, String description) {
        this.qos = qos;
        this.description = description;
    }

    public static Qos valueOf(MqttQos qos) {
        return Arrays.stream(values())
                     .filter(v -> qos == v.getMqttQos())
                     .findFirst()
                     .orElseThrow(() -> new IllegalArgumentException("Qos can not be matched."));
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static Qos fromJsonValue(int value) {
        if (value < 0 || value >= 2) {
            return null;
        }
        return values()[value];
    }

    @Override
    public String toString() {
        return "QoS " + qos.ordinal();
    }

    public MqttQos getMqttQos() {
        return qos;
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
