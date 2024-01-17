package org.correomqtt.business.connection;

import org.correomqtt.business.concurrent.SimpleProgressTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.eventbus.Subscribe;
import org.correomqtt.business.eventbus.SubscribeFilter;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientFactory;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;

import javax.net.ssl.SSLException;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

import static org.correomqtt.business.eventbus.SubscribeFilterNames.CONNECTION_ID;

public class ConnectTask extends SimpleProgressTask<ConnectionStateChangedEvent> {

    private final String connectionId;

    public ConnectTask(String connectionId) {
        this.connectionId = connectionId;
    }

    @Override
    protected void execute()  {
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        if (client == null) {
            CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(connectionId);
            connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));
            client = ConnectionHolder.getInstance().getClient(connectionId);
        }

        if (client.getState() == ConnectionState.DISCONNECTED_GRACEFUL || client.getState() == ConnectionState.DISCONNECTED_UNGRACEFUL) {
            try {
                client.connect();
            } catch (InterruptedException e) {
                throw new RuntimeException(e);
            } catch (ExecutionException e) {
                throw new RuntimeException(e);
            } catch (TimeoutException e) {
                throw new RuntimeException(e);
            } catch (SSLException e) {
                throw new RuntimeException(e);
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
