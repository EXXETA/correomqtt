package com.exxeta.correomqtt.business.model;

import com.exxeta.correomqtt.gui.model.GenericCellModel;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

public enum Lwt implements GenericCellModel {
    OFF("lwtEnumOff"),
    ON("lwtEnumActive");

    private final String lwt;

    Lwt(String lwt) {
        this.lwt = lwt;
    }

    public String getLwt() {
        return lwt;
    }

    @Override
    public String toString() { return lwt; }

    @SuppressWarnings("unused")
    @JsonCreator
    public static Lwt fromJsonValue(String value) {
        try {
            return Lwt.valueOf(value);
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
        return lwt;
    }
}
