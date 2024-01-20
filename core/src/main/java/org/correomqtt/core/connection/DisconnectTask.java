package org.correomqtt.core.connection;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.utils.ConnectionHolder;

public class DisconnectTask extends SimpleTask {

    private final ConnectionHolder connectionHolder;
    private final String connectionId;

    @AssistedInject
    public DisconnectTask(ConnectionHolder connectionHolder, @Assisted String connectionId) {
        this.connectionHolder = connectionHolder;
        this.connectionId = connectionId;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = connectionHolder.getClient(connectionId);
        client.disconnect();
    }


    @Override
    protected void successHook() {
        connectionHolder.getConnection(connectionId).setClient(null); // TODO extract
    }
}
