package org.correomqtt.core.connection;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.utils.ConnectionManager;

public class DisconnectTask extends SimpleTask {

    private final ConnectionManager connectionManager;
    private final String connectionId;

    @AssistedFactory
    public interface Factory {
        DisconnectTask create(String connectionId);
    }

    @AssistedInject
    public DisconnectTask(ConnectionManager connectionManager, @Assisted String connectionId) {
        this.connectionManager = connectionManager;
        this.connectionId = connectionId;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = connectionManager.getClient(connectionId);
        client.disconnect();
    }


    @Override
    protected void successHook() {
        connectionManager.getConnection(connectionId).setClient(null); // TODO extract
    }
}
