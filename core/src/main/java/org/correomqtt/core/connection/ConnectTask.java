package org.correomqtt.core.connection;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.di.Observes;
import org.correomqtt.di.ObservesFilter;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.mqtt.CorreoMqttClientFactory;
import org.correomqtt.core.utils.ConnectionManager;
import org.correomqtt.core.utils.CorreoMqttConnection;

import javax.net.ssl.SSLException;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

import static org.correomqtt.core.events.ObservesFilterNames.CONNECTION_ID;

@DefaultBean
public class ConnectTask extends SimpleProgressTask<ConnectionStateChangedEvent> {

    private final ConnectionManager connectionManager;
    private final CorreoMqttClientFactory correoMqttClientFactory;
    private final String connectionId;


    @Inject
    public ConnectTask(ConnectionManager connectionManager,
                       CorreoMqttClientFactory correoMqttClientFactory,
                       SoyEvents soyEvents,
                       @Assisted String connectionId) {
        super(soyEvents);
        this.connectionManager = connectionManager;
        this.correoMqttClientFactory = correoMqttClientFactory;
        this.connectionId = connectionId;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = connectionManager.getClient(connectionId);
        if (client == null) {
            CorreoMqttConnection connection = connectionManager.getConnection(connectionId);
            connection.setClient(correoMqttClientFactory.createClient(connection.getConfigDTO()));
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

    public void onConnectionStateChanged(@Observes ConnectionStateChangedEvent event) {
        reportProgress(event);
    }

    @ObservesFilter(CONNECTION_ID)
    public String getConnectionId() {
        return connectionId;
    }

}
