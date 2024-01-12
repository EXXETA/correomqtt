package org.correomqtt.business.connection;

import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;

public class DisconnectTask extends SimpleTask {

    private final String connectionId;

    public DisconnectTask(String connectionId) {
        this.connectionId = connectionId;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        client.disconnect();
    }


    @Override
    protected void successHook() {
        ConnectionHolder.getInstance().getConnection(connectionId).setClient(null); // TODO extract
    }
}
