package org.correomqtt.business.model;

import org.correomqtt.gui.model.GenericCellModel;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

public enum Proxy implements GenericCellModel {
    OFF("proxyEnumOff"),
    SSH("proxyEnumSsh");

    private final String proxy;

    Proxy(String proxy) {
        this.proxy = proxy;
    }

    public String getProxy() {
        return proxy;
    }

    @Override
    public String toString() {
        return proxy;
    }

    @SuppressWarnings("unused")
    @JsonCreator
    public static Proxy fromJsonValue(String value) {
        try {
            return Proxy.valueOf(value);
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
        return proxy;
    }
}
