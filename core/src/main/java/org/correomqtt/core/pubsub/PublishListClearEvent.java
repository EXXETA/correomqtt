package org.correomqtt.core.pubsub;

import lombok.AllArgsConstructor;
import org.correomqtt.di.Event;
import org.correomqtt.di.ObservesFilter;

import static org.correomqtt.core.events.ObservesFilterNames.CONNECTION_ID;

@AllArgsConstructor
public class PublishListClearEvent implements Event {

    private String connectionId;

    @ObservesFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }
}
