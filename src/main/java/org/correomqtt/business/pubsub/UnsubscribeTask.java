package org.correomqtt.business.pubsub;

import org.correomqtt.business.concurrent.ConnectionTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;

public class UnsubscribeTask extends ConnectionTask<Void, Void> {

    private final SubscriptionDTO subscriptionDTO;

    public UnsubscribeTask(String connectionId, SubscriptionDTO subscriptionDTO) {
        super(connectionId);
        this.subscriptionDTO = subscriptionDTO;
    }

    @Override
    protected Void execute() throws Exception {
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        client.unsubscribe(subscriptionDTO);
        EventBus.fireAsync(new UnsubscribeEvent(connectionId,subscriptionDTO));
        return null;
    }

    @Override
    protected void error(Void error, Throwable ex){
        EventBus.fireAsync(new UnsubscribeFailedEvent(connectionId,subscriptionDTO));
    }
}
