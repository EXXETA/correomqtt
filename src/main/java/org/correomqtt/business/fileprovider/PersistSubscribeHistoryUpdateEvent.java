package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.CONNECTION_ID;

public record PersistSubscribeHistoryUpdateEvent(String connectionId) implements Event {

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }
}
