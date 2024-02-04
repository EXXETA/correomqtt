package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Event;
import org.correomqtt.di.ObservesFilter;

import static org.correomqtt.core.events.ObservesFilterNames.CONNECTION_ID;

public record PersistSubscribeHistoryUpdateEvent(String connectionId) implements Event {

    @ObservesFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }
}
