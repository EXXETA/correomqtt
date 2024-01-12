package org.correomqtt.business.utils;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

public class LoggerUtils {

    private LoggerUtils(){
        // private constructor
    }

    public static Marker getConnectionMarker(String connectionId) {
        ConnectionConfigDTO connectionConfig = ConnectionHolder.getInstance().getConfig(connectionId);
        if (connectionConfig == null) {
            return MarkerFactory.getMarker("Unknown");
        }
        return MarkerFactory.getMarker(connectionConfig.getName());
    }
}
