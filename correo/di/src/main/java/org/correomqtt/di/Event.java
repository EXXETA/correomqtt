package org.correomqtt.di;

public interface Event {
    default boolean isLogable() {
        return true;
    }
}
