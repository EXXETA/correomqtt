package org.correomqtt.core.connection;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.mqtt.CorreoMqttClientFactory;
import org.correomqtt.core.pubsub.SubscribeTask;
import org.correomqtt.core.pubsub.SubscribeTaskFactory;
import org.correomqtt.core.utils.ConnectionHolder;
import org.correomqtt.core.utils.CorreoMqttConnection;

import java.util.Set;

public class ReconnectTask extends NoProgressTask<Void, Void> {

    private final SubscribeTaskFactory subscribeTaskFactory;
    private final ConnectionHolder connectionHolder;
    private final String connectionId;

    @AssistedInject
    public ReconnectTask(SubscribeTaskFactory subscribeTaskFactory, ConnectionHolder connectionHolder, @Assisted String connectionId) {
        this.subscribeTaskFactory = subscribeTaskFactory;
        this.connectionHolder = connectionHolder;
        this.connectionId = connectionId;
    }

    @Override
    protected Void execute() throws Exception {
        CorreoMqttConnection connection = connectionHolder.getConnection(connectionId);
        Set<SubscriptionDTO> existingSubscriptions = connection.getClient().getSubscriptions();

        connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));
        CorreoMqttClient client = connectionHolder.getClient(connectionId);
        client.connect();

        existingSubscriptions.forEach(subscriptionDTO -> subscribeTaskFactory.create(connectionId, subscriptionDTO).run());

        return null;
    }
}
