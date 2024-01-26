package org.correomqtt.core.pubsub;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.utils.ConnectionManager;

public class UnsubscribeTask extends SimpleTask {

    private final ConnectionManager connectionManager;
    private final EventBus eventBus;
    private final String connectionId;
    private final SubscriptionDTO subscriptionDTO;

    @AssistedFactory
    public interface Factory {
        UnsubscribeTask create(String connectionId, SubscriptionDTO subscriptionDTO);
    }

    @AssistedInject
    public UnsubscribeTask(ConnectionManager connectionManager,
                           EventBus eventBus,
                           @Assisted String connectionId,
                           @Assisted SubscriptionDTO subscriptionDTO) {
        super(eventBus);
        this.connectionManager = connectionManager;
        this.eventBus = eventBus;
        this.connectionId = connectionId;
        this.subscriptionDTO = subscriptionDTO;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = connectionManager.getClient(connectionId);
        client.unsubscribe(subscriptionDTO);
        eventBus.fireAsync(new UnsubscribeEvent(connectionId, subscriptionDTO));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        eventBus.fireAsync(new UnsubscribeFailedEvent(connectionId, subscriptionDTO));
    }

}
