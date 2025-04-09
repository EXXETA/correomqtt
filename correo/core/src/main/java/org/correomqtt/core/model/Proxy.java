package org.correomqtt.core.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public enum Proxy implements GenericTranslatable {
    OFF("proxyEnumOff"),
    SSH("proxyEnumSsh");

    private static final Logger LOGGER = LoggerFactory.getLogger(Proxy.class);

    private final String labelTranslationKey;

    Proxy(String labelTranslationKey) {
        this.labelTranslationKey = labelTranslationKey;
    }

    @Override
    public String toString() {
        return labelTranslationKey;
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static Proxy fromJsonValue(String value) {
        try {
            return Proxy.valueOf(value);
        } catch (IllegalArgumentException iae) {
            LOGGER.warn("Exception parsing json value.", iae);
            return OFF;
        }
    }

    @SuppressWarnings("unused")
    @JsonValue
    public String toJsonValue() {
        return name();
    }

    @Override
    public String getLabelTranslationKey() {
        return labelTranslationKey;
    }
}
