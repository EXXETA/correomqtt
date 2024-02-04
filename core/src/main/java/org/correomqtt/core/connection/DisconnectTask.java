package org.correomqtt.core.connection;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.utils.ConnectionManager;

@DefaultBean
public class DisconnectTask extends SimpleTask {

    private final ConnectionManager connectionManager;
    private final String connectionId;

    @Inject
    public DisconnectTask(ConnectionManager connectionManager,
                          EventBus eventBus,
                          @Assisted String connectionId) {
        super(eventBus);
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
