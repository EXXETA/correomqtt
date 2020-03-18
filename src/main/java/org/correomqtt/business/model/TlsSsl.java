package org.correomqtt.business.model;

import org.correomqtt.gui.model.GenericCellModel;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

public enum TlsSsl implements GenericCellModel {
    OFF("tlsEnumOff"),
    KEYSTORE("tlsEnumKeystore");

    private final String tls;

    TlsSsl(String tls) {
        this.tls = tls;
    }

    public String getTls() {
        return tls;
    }

    @Override
    public String toString() {
        return tls;
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static TlsSsl fromJsonValue(String value) {
        try {
            return TlsSsl.valueOf(value);
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
        return tls;
    }
}
