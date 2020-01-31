package com.exxeta.correomqtt.plugin.spi;

import org.jdom2.Element;
import org.pf4j.ExtensionPoint;

public interface BaseExtensionPoint extends ExtensionPoint {

    default void onConfigReceived(Element config) {}
}
