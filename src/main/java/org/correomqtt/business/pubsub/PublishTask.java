package org.correomqtt.business.pubsub;

import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.business.utils.ConnectionHolder;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.gui.utils.AlertHelper;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.OutgoingMessageHook;
import org.correomqtt.plugin.spi.OutgoingMessageHookDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

import static org.correomqtt.business.utils.LoggerUtils.getConnectionMarker;

public class PublishTask extends SimpleTask {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishTask.class);


    private final String connectionId;
    private final MessageDTO messageDTO;

    public PublishTask(String connectionId, MessageDTO messageDTO) {
        this.connectionId = connectionId;
        this.messageDTO = messageDTO;
    }

    @Override
    protected void execute() {
        LOGGER.info(getConnectionMarker(connectionId), "Start publishing to topic: {}", messageDTO.getTopic());
        CorreoMqttClient client = ConnectionHolder.getInstance().getClient(connectionId);
        MessageDTO manipulatedMessageDTO = executeOnPublishMessageExtensions(connectionId, messageDTO);
        try {
            client.publish(manipulatedMessageDTO);
            EventBus.fireAsync(new PublishEvent(connectionId, manipulatedMessageDTO));
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            AlertHelper.unexpectedAlert(e);
        } catch (ExecutionException | TimeoutException e) {
            AlertHelper.unexpectedAlert(e);
        }
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
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
                LOGGER.info(getConnectionMarker(connectionId), "[HOOK] Manipulated outgoing message on {} with {}", messageDTO.getTopic(), p.getClass().getName());
                messageExtensionDTO = p.onPublishMessage(connectionId, messageExtensionDTO);
            }
        }
        return MessageTransformer.mergeDTO(messageExtensionDTO, messageDTO);
    }
}
