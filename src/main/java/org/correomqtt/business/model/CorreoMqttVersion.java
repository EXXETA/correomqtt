package com.exxeta.correomqtt.business.model;

import com.exxeta.correomqtt.gui.model.GenericCellModel;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import com.hivemq.client.mqtt.MqttVersion;

public enum CorreoMqttVersion implements GenericCellModel {
    MQTT_3_1_1(MqttVersion.MQTT_3_1_1, "MQTT v3.1.1"),
    MQTT_5_0(MqttVersion.MQTT_5_0, "MQTT v5.0");

    private MqttVersion version;
    private String description;

    CorreoMqttVersion(MqttVersion version, String description) {
        this.version = version;
        this.description = description;
    }

    public MqttVersion getVersion() {
        return version;
    }

    public String getDescription() {
        return description;
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static CorreoMqttVersion fromJsonValue(String value) {
        try {
            return CorreoMqttVersion.valueOf(value);
        } catch (IllegalArgumentException iae) {
            //TODO: Log
            return MQTT_3_1_1;
        }
    }

    @SuppressWarnings("unused")
    @JsonValue
    public String toJsonValue() {
        return name();
    }

    @Override
    public String toString() {
        return description;
    }

    @Override
    public String getLabelTranslationKey() {
        return null;
    }
}
