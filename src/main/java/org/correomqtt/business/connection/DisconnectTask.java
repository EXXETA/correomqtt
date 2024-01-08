package org.correomqtt.business.connection;

import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;

public class DisconnectTask extends NoProgressTask<Void, Void> {

    private final String connectionId;

    public DisconnectTask(String connectionId) {
        this.connectionId = connectionId;
    }

    @Override
    protected Void execute() throws Exception {
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        client.disconnect();
        return null;
    }


    @Override
    protected void success(Void ignored) {
        ConnectionHolder.getInstance().getConnection(connectionId).setClient(null); // TODO extract
    }
}
