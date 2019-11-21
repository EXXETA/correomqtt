package com.exxeta.correomqtt.business.model;

import com.exxeta.correomqtt.gui.model.GenericCellModel;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

public enum Auth implements GenericCellModel {
    OFF("authEnumOff"),
    PASSWORD("authEnumKeyfile"),
    KEYFILE("authEnumPassword");

    private final String auth;

    Auth(String auth) {
        this.auth = auth;
    }

    public String getAuth() {
        return auth;
    }

    @Override
    public String toString() {
        return auth;
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static Auth fromJsonValue(String value) {
        try {
            return Auth.valueOf(value);
        } catch (IllegalArgumentException iae) {
            //TODO: Log
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
        return auth;
    }
}
