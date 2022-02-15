package org.correomqtt.plugin.spi;

import com.fasterxml.jackson.databind.JsonNode;
import org.pf4j.ExtensionPoint;

public interface BaseExtensionPoint extends ExtensionPoint {

    default void onConfigReceived(JsonNode config) {}
}
