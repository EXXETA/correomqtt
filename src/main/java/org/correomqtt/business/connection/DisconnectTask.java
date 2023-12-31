package org.correomqtt.business.connection;

import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;

public class DisconnectTask extends Task<Void, Void> {

    private final String connectionId;

    public DisconnectTask(String connectionId) {
        this.connectionId = connectionId;
    }

    @Override
    protected void before() {
        EventBus.fireAsync(new DisconnectStartedEvent(connectionId));
    }

    @Override
    protected Void execute() throws Exception {
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        client.disconnect(true);

        EventBus.fireAsync(new DisconnectEvent(connectionId));
        return null;
    }

    @Override
    protected void error(Throwable throwable) {
        EventBus.fireAsync(new DisconnectFailedEvent(connectionId, throwable));

    }

    @Override
    protected void success(Void ignored) {
        ConnectionHolder.getInstance().getConnection(connectionId).setClient(null);
    }
}
