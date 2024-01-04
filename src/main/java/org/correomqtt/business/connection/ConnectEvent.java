package org.correomqtt.business.connection;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;

@AllArgsConstructor
@Getter
public class ConnectEvent implements Event {
    private String connectionId;

    @SubscribeFilter(value ="connectionId")
    public String getConnectionId(){
        return this.connectionId;
    }
}
