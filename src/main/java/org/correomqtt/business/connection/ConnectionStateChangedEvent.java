package org.correomqtt.business.connection;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.business.eventbus.Event;
import org.correomqtt.business.eventbus.SubscribeFilter;

@AllArgsConstructor
@Getter
public class ConnectionStateChangedEvent implements Event {
    private String connectionId;
    private ConnectionState state;
    private int retries;
    private int maxRetries;

    @SubscribeFilter(value = "connectionId")
    public String getConnectionId() {
        return this.connectionId;
    }
}
