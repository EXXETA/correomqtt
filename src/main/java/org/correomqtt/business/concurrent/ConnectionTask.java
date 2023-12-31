package org.correomqtt.business.concurrent;

import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.FrontendBinding;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;

public abstract class ConnectionTask<T, E> extends Task<T,E>{

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
