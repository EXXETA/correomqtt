package org.correomqtt.business.pubsub;

import lombok.AllArgsConstructor;
import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.CONNECTION_ID;

@AllArgsConstructor
public class PublishListClearEvent implements Event {

    private String connectionId;

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }
}
