package org.correomqtt.core.connection;

import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.di.Event;
import org.correomqtt.di.ObservesFilter;

@AllArgsConstructor
@Getter
public class ConnectionStateChangedEvent implements Event {
    private String connectionId;
    private ConnectionState state;
    private int retries;
    private int maxRetries;

    @ObservesFilter(value = "connectionId")
    public String getConnectionId() {
        return this.connectionId;
    }
}
