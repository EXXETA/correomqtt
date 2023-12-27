package org.correomqtt.business.dispatcher;

import org.correomqtt.business.utils.ConnectionHolder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MarkerFactory;

import java.util.function.Consumer;

public abstract class BaseConnectionDispatcher<T extends BaseConnectionObserver> extends BaseDispatcher<T>  {

    private static final Logger LOGGER = LoggerFactory.getLogger(BaseConnectionDispatcher.class);

    void triggerFiltered(String connectionId, Consumer<T> trigger) {
        final String callerString = getCallerString();

        if (LOGGER.isDebugEnabled()) {
            LOGGER.debug(MarkerFactory.getMarker(ConnectionHolder.getInstance().getConfig(connectionId).getName()),
                         "Trigger with connectionId {}: {}",
                         connectionId, callerString);
        }
        observer.stream()
                .filter(o -> o.getConnectionId() != null)
                .filter(o -> o.getConnectionId().equals(connectionId))
                .toList()
                .forEach(o -> {
                    if (LOGGER.isTraceEnabled()) {
                        LOGGER.trace(MarkerFactory.getMarker(ConnectionHolder.getInstance().getConfig(connectionId).getName()),
                                     "Trigger with connectionId {}: {} -> {}",
                                     connectionId, callerString, o.getClass().getSimpleName());
                    }
                    trigger.accept(o);
                });
    }
}
