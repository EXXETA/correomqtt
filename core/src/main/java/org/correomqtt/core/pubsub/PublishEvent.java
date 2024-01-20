package org.correomqtt.core.pubsub;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.core.eventbus.Event;
import org.correomqtt.core.eventbus.SubscribeFilter;
import org.correomqtt.core.model.MessageDTO;

import static org.correomqtt.core.eventbus.SubscribeFilterNames.CONNECTION_ID;

@AllArgsConstructor
@Getter
public class PublishEvent implements Event {
    private String connectionId;
    private MessageDTO messageDTO;

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId(){
        return connectionId;
    }
}
