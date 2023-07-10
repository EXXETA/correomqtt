package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import com.hivemq.client.mqtt.MqttVersion;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public enum CorreoMqttVersion implements GenericTranslatable {
    MQTT_3_1_1(MqttVersion.MQTT_3_1_1, "MQTT v3.1.1"),
    MQTT_5_0(MqttVersion.MQTT_5_0, "MQTT v5.0");

    private static final Logger LOGGER = LoggerFactory.getLogger(CorreoMqttVersion.class);

    private final MqttVersion version;
    private final String description;

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
            LOGGER.warn("Exception reading version", iae);
            return MQTT_5_0;
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
