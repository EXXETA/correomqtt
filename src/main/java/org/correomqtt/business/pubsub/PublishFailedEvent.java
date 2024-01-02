package org.correomqtt.business.pubsub;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.model.MessageDTO;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.CONNECTION_ID;

@AllArgsConstructor
@Getter
public class PublishFailedEvent implements Event {
    private String connectionId;
    private MessageDTO messageDTO;

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId(){
        return connectionId;
    }
}
