package org.correomqtt.core.connection;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.eventbus.Subscribe;
import org.correomqtt.core.eventbus.SubscribeFilter;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.mqtt.CorreoMqttClientFactory;
import org.correomqtt.core.utils.ConnectionManager;
import org.correomqtt.core.utils.CorreoMqttConnection;

import javax.net.ssl.SSLException;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

import static org.correomqtt.core.eventbus.SubscribeFilterNames.CONNECTION_ID;

public class ConnectTask extends SimpleProgressTask<ConnectionStateChangedEvent> {

    private final ConnectionManager connectionManager;
    private final String connectionId;


    @AssistedFactory
    public interface Factory {
        ConnectTask create(String connectionId);
    }

    @AssistedInject
    public ConnectTask(ConnectionManager connectionManager, @Assisted String connectionId) {
        this.connectionManager = connectionManager;
        this.connectionId = connectionId;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = connectionManager.getClient(connectionId);
        if (client == null) {
            CorreoMqttConnection connection = connectionManager.getConnection(connectionId);
            connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));
            client = connectionManager.getClient(connectionId);
        }

        if (client.getState() == ConnectionState.DISCONNECTED_GRACEFUL || client.getState() == ConnectionState.DISCONNECTED_UNGRACEFUL) {
            try {
                client.connect();
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new TaskException(e);
            } catch (ExecutionException | TimeoutException | SSLException e) {
                throw new TaskException(e);
            }
        }
    }

    @Override
    protected void beforeHook() {
        EventBus.register(this);
    }

    @Override
    protected void finalHook() {
        EventBus.unregister(this);
    }

    public void onConnectionStateChanged(@Subscribe ConnectionStateChangedEvent event) {
        reportProgress(event);
    }

    @SubscribeFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }

}
