package org.correomqtt.core.pubsub;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.utils.ConnectionHolder;

public class UnsubscribeTask extends SimpleTask {

    private final ConnectionHolder connectionHolder;
    private final String connectionId;
    private final SubscriptionDTO subscriptionDTO;

    @AssistedInject
    public UnsubscribeTask(ConnectionHolder connectionHolder, @Assisted String connectionId, @Assisted SubscriptionDTO subscriptionDTO) {
        this.connectionHolder = connectionHolder;
        this.connectionId = connectionId;
        this.subscriptionDTO = subscriptionDTO;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = connectionHolder.getClient(connectionId);
        client.unsubscribe(subscriptionDTO);
        EventBus.fireAsync(new UnsubscribeEvent(connectionId, subscriptionDTO));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new UnsubscribeFailedEvent(connectionId, subscriptionDTO));
    }
}
