package org.correomqtt.business.pubsub;

import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
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

import static org.correomqtt.business.utils.LoggerUtils.getConnectionMarker;

public class SubscribeTask extends SimpleTask {

    private static final Logger LOGGER = LoggerFactory.getLogger(SubscribeTask.class);


    private final String connectionId;
    private final SubscriptionDTO subscriptionDTO;

    public SubscribeTask(String connectionId, SubscriptionDTO subscriptionDTO) {
        this.connectionId = connectionId;
        this.subscriptionDTO = subscriptionDTO;
    }

    @Override
    protected void execute() throws Exception {
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        client.subscribe(subscriptionDTO, this::onIncomingMessage);
        EventBus.fireAsync(new SubscribeEvent(connectionId, subscriptionDTO));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new SubscribeFailedEvent(connectionId, subscriptionDTO));
    }

    private void onIncomingMessage(MessageDTO messageDTO) {

        MessageDTO manipulatedMessageDTO = executeOnMessageIncomingExtensions(messageDTO);
        EventBus.fireAsync(new IncomingMessageEvent(connectionId, manipulatedMessageDTO, subscriptionDTO));
    }

    private MessageDTO executeOnMessageIncomingExtensions(MessageDTO messageDTO) {

        //TODO plugin stuff in business ... bäääh ... solve via EventBus
        MessageExtensionDTO messageExtensionDTO = new MessageExtensionDTO(messageDTO);
        for (IncomingMessageHook<?> p : PluginManager.getInstance().getIncomingMessageHooks()) {
            IncomingMessageHookDTO config = p.getConfig();
            if (config != null && config.isEnableIncoming() && (config.getIncomingTopicFilter() == null ||
                    config.getIncomingTopicFilter()
                            .stream()
                            .anyMatch(tp -> MqttTopicFilter.of(tp)
                                    .matches(MqttTopic.of(messageDTO.getTopic()))
                            )
            )) {
                LOGGER.info(getConnectionMarker(connectionId), "[HOOK] Manipulated incoming message on {} with {}", messageDTO.getTopic(), p.getClass().getName());
                messageExtensionDTO = p.onMessageIncoming(connectionId, messageExtensionDTO);
            }
        }
        return MessageTransformer.mergeDTO(messageExtensionDTO, messageDTO);
    }
}
