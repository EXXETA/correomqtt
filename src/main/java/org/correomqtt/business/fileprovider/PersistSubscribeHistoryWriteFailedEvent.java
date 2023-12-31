package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;

import java.io.IOException;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.CONNECTION_ID;

public record PersistSubscribeHistoryWriteFailedEvent(String connectionId, Throwable throwable) implements Event {

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }
}
