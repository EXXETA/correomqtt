package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public enum TlsSsl implements GenericTranslatable {
    OFF("tlsEnumOff"),
    KEYSTORE("tlsEnumKeystore");

    private static final Logger LOGGER = LoggerFactory.getLogger(TlsSsl.class);

    private final String labelTranslationKey;

    TlsSsl(String labelTranslationKey) {
        this.labelTranslationKey = labelTranslationKey;
    }


    @Override
    public String toString() {
        return labelTranslationKey;
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static TlsSsl fromJsonValue(String value) {
        try {
            return TlsSsl.valueOf(value);
        } catch (IllegalArgumentException iae) {
            LOGGER.warn("Exception reading from json value.", iae);
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
