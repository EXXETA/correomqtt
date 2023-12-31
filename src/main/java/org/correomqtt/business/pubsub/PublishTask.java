package org.correomqtt.business.pubsub;

import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import org.correomqtt.business.concurrent.ConnectionTask;
import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.connection.ConnectEvent;
import org.correomqtt.business.connection.ConnectFailedEvent;
import org.correomqtt.business.connection.ConnectStartedEvent;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.mqtt.CorreoMqttClientFactory;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.business.utils.CorreoMqttConnection;
import org.correomqtt.gui.controller.PublishViewController;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.OutgoingMessageHook;
import org.correomqtt.plugin.spi.OutgoingMessageHookDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class PublishTask extends ConnectionTask<Void, Void> {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishTask.class);


    private final MessageDTO messageDTO;

    public PublishTask(String connectionId, MessageDTO messageDTO) {
        super(connectionId);
        this.messageDTO = messageDTO;
    }

    @Override
    protected Void execute() throws Exception {
        LOGGER.info(getConnectionMarker(), "Start publishing to topic: {}", messageDTO.getTopic());
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        MessageDTO manipulatedMessageDTO = executeOnPublishMessageExtensions(connectionId, messageDTO);
        client.publish(manipulatedMessageDTO);
        EventBus.fireAsync(new PublishEvent(connectionId, manipulatedMessageDTO));
        return null;
    }

    @Override
    protected void error(Void error, Throwable ex) {
        EventBus.fireAsync(new PublishFailedEvent(connectionId, messageDTO));
    }


    private MessageDTO executeOnPublishMessageExtensions(String connectionId, MessageDTO messageDTO) {
        MessageExtensionDTO messageExtensionDTO = new MessageExtensionDTO(messageDTO);
        for (OutgoingMessageHook<?> p : PluginManager.getInstance().getOutgoingMessageHooks()) {
            OutgoingMessageHookDTO config = p.getConfig();
            if (config != null && config.isEnableOutgoing() && (config.getOutgoingTopicFilter() == null ||
                    config.getOutgoingTopicFilter()
                            .stream()
                            .anyMatch(tp -> MqttTopicFilter.of(tp)
                                    .matches(MqttTopic.of(messageDTO.getTopic()))
                            )
            )) {
                LOGGER.info(getConnectionMarker(), "[HOOK] Manipulated outgoing message on {} with {}", messageDTO.getTopic(), p.getClass().getName());
                messageExtensionDTO = p.onPublishMessage(connectionId, messageExtensionDTO);
            }
        }
        return MessageTransformer.mergeDTO(messageExtensionDTO, messageDTO);
    }
}
