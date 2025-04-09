package org.correomqtt.core.pubsub;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.utils.ConnectionManager;

@DefaultBean
public class UnsubscribeTask extends SimpleTask {

    private final ConnectionManager connectionManager;
    private final SoyEvents soyEvents;
    private final String connectionId;
    private final SubscriptionDTO subscriptionDTO;

    @Inject
    public UnsubscribeTask(ConnectionManager connectionManager,
                           SoyEvents soyEvents,
                           @Assisted String connectionId,
                           @Assisted SubscriptionDTO subscriptionDTO) {
        super(soyEvents);
        this.connectionManager = connectionManager;
        this.soyEvents = soyEvents;
        this.connectionId = connectionId;
        this.subscriptionDTO = subscriptionDTO;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = connectionManager.getClient(connectionId);
        client.unsubscribe(subscriptionDTO);
        soyEvents.fireAsync(new UnsubscribeEvent(connectionId, subscriptionDTO));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        soyEvents.fireAsync(new UnsubscribeFailedEvent(connectionId, subscriptionDTO));
    }

}
