package org.correomqtt.business.connection;

import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientFactory;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;

public class ConnectTask extends SimpleTask {

    private final String connectionId;

    public ConnectTask(String connectionId) {
        this.connectionId = connectionId;
    }

    @Override
    protected void execute() throws Exception {

        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        if (client == null) {
            CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(connectionId);
            connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));
            client = ConnectionHolder.getInstance().getClient(connectionId);
        }

        if (client.getState() == ConnectionState.DISCONNECTED_GRACEFUL ||
                client.getState() == ConnectionState.DISCONNECTED_UNGRACEFUL) {
            client.connect();
        }
    }
}
