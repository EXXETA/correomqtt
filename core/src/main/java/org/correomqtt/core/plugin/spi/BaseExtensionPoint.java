package org.correomqtt.core.plugin.spi;

import org.pf4j.ExtensionPoint;

public interface BaseExtensionPoint<T> extends ExtensionPoint {

    default Class<T> getConfigClass() {
        return null;
    }

    default T getConfig(){
        return null;
    }

    default void onConfigReceived(T config) {}

    default String getConfigNamespace(){
        return null;
    }
}
