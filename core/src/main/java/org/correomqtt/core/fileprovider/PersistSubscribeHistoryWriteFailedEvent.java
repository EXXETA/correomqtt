package org.correomqtt.core.fileprovider;

import org.correomqtt.core.eventbus.Event;
import org.correomqtt.core.eventbus.SubscribeFilter;

import static org.correomqtt.core.eventbus.SubscribeFilterNames.CONNECTION_ID;

public record PersistSubscribeHistoryWriteFailedEvent(String connectionId, Throwable throwable) implements Event {

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }
}
