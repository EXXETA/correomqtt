package org.correomqtt.business.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public enum Lwt implements GenericTranslatable{
    OFF("lwtEnumOff"),
    ON("lwtEnumActive");

    private static final Logger LOGGER = LoggerFactory.getLogger(Lwt.class);

    private final String labelTranslationKey;

    Lwt(String labelTranslationKey) {
        this.labelTranslationKey = labelTranslationKey;
    }

    @Override
    public String toString() {
        return labelTranslationKey;
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static Lwt fromJsonValue(String value) {
        try {
            return Lwt.valueOf(value);
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

    public String getLabelTranslationKey() {
        return labelTranslationKey;
    }
}
