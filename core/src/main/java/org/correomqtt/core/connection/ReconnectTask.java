package org.correomqtt.core.connection;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.mqtt.CorreoMqttClientFactory;
import org.correomqtt.core.pubsub.SubscribeTask;
import org.correomqtt.core.utils.ConnectionManager;
import org.correomqtt.core.utils.CorreoMqttConnection;

import java.util.Set;

public class ReconnectTask extends NoProgressTask<Void, Void> {

    private final SubscribeTask.Factory subscribeTaskFactory;
    private final ConnectionManager connectionManager;
    private final String connectionId;

    @AssistedFactory
    public interface Factory {
        ReconnectTask create(String connectionId);
    }

    @AssistedInject
    public ReconnectTask(SubscribeTask.Factory subscribeTaskFactory, ConnectionManager connectionManager, @Assisted String connectionId) {
        this.subscribeTaskFactory = subscribeTaskFactory;
        this.connectionManager = connectionManager;
        this.connectionId = connectionId;
    }

    @Override
    protected Void execute() throws Exception {
        CorreoMqttConnection connection = connectionManager.getConnection(connectionId);
        Set<SubscriptionDTO> existingSubscriptions = connection.getClient().getSubscriptions();

        connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));
        CorreoMqttClient client = connectionManager.getClient(connectionId);
        client.connect();

        existingSubscriptions.forEach(subscriptionDTO -> subscribeTaskFactory.create(connectionId, subscriptionDTO).run());

        return null;
    }
}
