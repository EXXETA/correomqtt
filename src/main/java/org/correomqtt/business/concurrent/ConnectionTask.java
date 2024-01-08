package org.correomqtt.business.concurrent;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.utils.ConnectionHolder;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

public abstract class ConnectionTask<T, E> extends NoProgressTask<T,E> {

    protected final String connectionId;

    protected ConnectionTask(String connectionId){
        this.connectionId = connectionId;
    }

    protected Marker getConnectionMarker() {
        ConnectionConfigDTO connectionConfig = ConnectionHolder.getInstance().getConfig(connectionId);
        if (connectionConfig == null) {
            return MarkerFactory.getMarker("Unknown");
        }
        return MarkerFactory.getMarker(connectionConfig.getName());
    }
}
