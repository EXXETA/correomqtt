package org.correomqtt.core.connection;

import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.mqtt.CorreoMqttClientFactory;
import org.correomqtt.core.pubsub.SubscribeTaskFactory;
import org.correomqtt.core.utils.ConnectionManager;
import org.correomqtt.core.utils.CorreoMqttConnection;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;

import java.util.Set;

@DefaultBean
public class ReconnectTask extends NoProgressTask<Void, Void> {

    private final SubscribeTaskFactory subscribeTaskFactory;
    private final CorreoMqttClientFactory correoMqttClientFactory;
    private final ConnectionManager connectionManager;
    private final String connectionId;


    @Inject
    public ReconnectTask(SubscribeTaskFactory subscribeTaskFactory,
                         SoyEvents soyEvents,
                         CorreoMqttClientFactory correoMqttClientFactory,
                         ConnectionManager connectionManager,
                         @Assisted String connectionId) {
        super(soyEvents);
        this.subscribeTaskFactory = subscribeTaskFactory;
        this.correoMqttClientFactory = correoMqttClientFactory;
        this.connectionManager = connectionManager;
        this.connectionId = connectionId;
    }

    @Override
    protected Void execute() throws Exception {
        CorreoMqttConnection connection = connectionManager.getConnection(connectionId);
        Set<SubscriptionDTO> existingSubscriptions = connection.getClient().getSubscriptions();

        connection.setClient(correoMqttClientFactory.createClient(connection.getConfigDTO()));
        CorreoMqttClient client = connectionManager.getClient(connectionId);
        client.connect();

        existingSubscriptions.forEach(subscriptionDTO -> subscribeTaskFactory.create(connectionId, subscriptionDTO).run());

        return null;
    }
}
