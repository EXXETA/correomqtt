package org.correomqtt.business.pubsub;

import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import org.correomqtt.business.concurrent.ConnectionTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.IncomingMessageHook;
import org.correomqtt.plugin.spi.IncomingMessageHookDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class UnsubscribeTask extends ConnectionTask<Void, Void> {

    private static final Logger LOGGER = LoggerFactory.getLogger(UnsubscribeTask.class);


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
