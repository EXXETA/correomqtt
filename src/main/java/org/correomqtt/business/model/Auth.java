package org.correomqtt.business.model;

import org.correomqtt.gui.model.GenericCellModel;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public enum Auth implements GenericCellModel {
    OFF("authEnumOff"),
    PASSWORD("authEnumPassword"),
    KEYFILE("authEnumKeyfile");

    private static final Logger LOGGER = LoggerFactory.getLogger(Auth.class);

    private final String labelTranslationKey;

    Auth(String labelTranslationKey) {
        this.labelTranslationKey = labelTranslationKey;
    }

    @Override
    public String toString() {
        return labelTranslationKey;
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static Auth fromJsonValue(String value) {
        try {
            return Auth.valueOf(value);
        } catch (IllegalArgumentException iae) {
            LOGGER.debug("Unknown auth value {}", value);
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
