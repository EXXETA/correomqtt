package org.correomqtt.business.pubsub;

import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;

public class UnsubscribeTask extends SimpleTask {

    private final String connectionId;
    private final SubscriptionDTO subscriptionDTO;

    public UnsubscribeTask(String connectionId, SubscriptionDTO subscriptionDTO) {
        this.connectionId = connectionId;
        this.subscriptionDTO = subscriptionDTO;
    }

    @Override
    protected void execute() {
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        client.unsubscribe(subscriptionDTO);
        EventBus.fireAsync(new UnsubscribeEvent(connectionId, subscriptionDTO));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new UnsubscribeFailedEvent(connectionId, subscriptionDTO));
    }
}
