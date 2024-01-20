package org.correomqtt.core.pubsub;

import lombok.AllArgsConstructor;
import org.correomqtt.core.eventbus.Event;
import org.correomqtt.core.eventbus.SubscribeFilter;

import static org.correomqtt.core.eventbus.SubscribeFilterNames.CONNECTION_ID;

@AllArgsConstructor
public class PublishListClearEvent implements Event {

    private String connectionId;

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }
}
