package org.correomqtt.business.pubsub;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.model.SubscriptionDTO;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.CONNECTION_ID;

@AllArgsConstructor
@Getter
public class UnsubscribeEvent implements Event {

    private String connectionId;
    private SubscriptionDTO subscriptionDTO;

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId(){
        return connectionId;
    }
}
