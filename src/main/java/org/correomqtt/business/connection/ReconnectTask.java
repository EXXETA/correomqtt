package org.correomqtt.business.connection;

import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientFactory;
import org.correomqtt.business.pubsub.SubscribeTask;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;

import java.util.Set;

public class ReconnectTask extends Task<Void, Void> {

    private final String connectionId;

    public ReconnectTask(String connectionId) {
        this.connectionId = connectionId;
    }

    @Override
    protected Void execute() throws Exception {
        CorreoMqttConnection connection = ConnectionHolder.getInstance().getConnection(connectionId);
        Set<SubscriptionDTO> existingSubscriptions = connection.getClient().getSubscriptions();

        connection.setClient(CorreoMqttClientFactory.createClient(connection.getConfigDTO()));
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        client.connect();

        existingSubscriptions.forEach(subscriptionDTO -> {
            new SubscribeTask(connectionId,subscriptionDTO).run();
        });

        EventBus.fireAsync(new ReconnectEvent(connectionId));

        return null;
    }
}
