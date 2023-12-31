package org.correomqtt.business.connection;

import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientFactory;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;

public class ConnectTask extends Task<Void, Void> {

    private final String connectionId;

    public ConnectTask(String connectionId) {
        this.connectionId = connectionId;
    }

    @Override
    protected void before() {
        EventBus.fireAsync(new ConnectStartedEvent(connectionId));
    }

    @Override
    protected Void execute() throws Exception {
        CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(connectionId);
        connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        client.connect();
        EventBus.fireAsync(new ConnectEvent(connectionId));
        return null;
    }

    @Override
    protected void error(Throwable t) {
        EventBus.fireAsync(new ConnectFailedEvent(connectionId, t));
    }

}
