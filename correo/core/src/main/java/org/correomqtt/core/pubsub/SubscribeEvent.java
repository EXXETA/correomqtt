package org.correomqtt.core.pubsub;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.di.Event;
import org.correomqtt.di.ObservesFilter;
import org.correomqtt.core.model.SubscriptionDTO;

import static org.correomqtt.core.events.ObservesFilterNames.CONNECTION_ID;

@AllArgsConstructor
@Getter
public class SubscribeEvent implements Event {
    private String connectionId;
    private SubscriptionDTO subscriptionDTO;

    @ObservesFilter(CONNECTION_ID)
    public String getConnectionId(){
        return connectionId;
    }
}
