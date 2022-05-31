package org.correomqtt.plugin.spi;

import org.pf4j.ExtensionPoint;

public interface BaseExtensionPoint<T> extends ExtensionPoint {

    default Class<T> getConfigClass() {
        return null;
    }

    default void onConfigReceived(T config) {}
}
