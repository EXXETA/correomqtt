package org.correomqtt.core.pubsub;

import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.mqtt.CorreoMqttClient;
import org.correomqtt.core.plugin.PluginManager;
import org.correomqtt.core.plugin.model.MessageExtensionDTO;
import org.correomqtt.core.plugin.spi.OutgoingMessageHook;
import org.correomqtt.core.plugin.spi.OutgoingMessageHookDTO;
import org.correomqtt.core.transformer.MessageExtensionTransformer;
import org.correomqtt.core.utils.ConnectionManager;
import org.correomqtt.core.utils.LoggerUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;


@DefaultBean
public class PublishTask extends SimpleTask {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishTask.class);

    private final PluginManager pluginManager;
    private final ConnectionManager connectionManager;
    private final LoggerUtils loggerUtils;
    private final EventBus eventBus;
    private final String connectionId;
    private final MessageDTO messageDTO;

    @Inject
    PublishTask(PluginManager pluginManager,
                ConnectionManager connectionManager,
                LoggerUtils loggerUtils,
                EventBus eventBus,
                @Assisted String connectionId,
                @Assisted MessageDTO messageDTO) {
        super(eventBus);
        this.pluginManager = pluginManager;
        this.connectionManager = connectionManager;
        this.loggerUtils = loggerUtils;
        this.eventBus = eventBus;
        this.connectionId = connectionId;
        this.messageDTO = messageDTO;
    }

    @Override
    protected void execute() {
        LOGGER.info(loggerUtils.getConnectionMarker(connectionId), "Start publishing to topic: {}", messageDTO.getTopic());
        CorreoMqttClient client = connectionManager.getClient(connectionId);
        MessageDTO manipulatedMessageDTO = executeOnPublishMessageExtensions(connectionId, messageDTO);
        try {
            client.publish(manipulatedMessageDTO);
            eventBus.fireAsync(new PublishEvent(connectionId, manipulatedMessageDTO));
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new TaskException(e);
        } catch (ExecutionException | TimeoutException e) {
            throw new TaskException(e);
        }
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        eventBus.fireAsync(new PublishFailedEvent(connectionId, messageDTO));
    }

    private MessageDTO executeOnPublishMessageExtensions(String connectionId, MessageDTO messageDTO) {
        MessageExtensionDTO messageExtensionDTO = new MessageExtensionDTO(messageDTO);
        for (OutgoingMessageHook<?> p : pluginManager.getOutgoingMessageHooks()) {
            OutgoingMessageHookDTO config = p.getConfig();
            if (config != null && config.isEnableOutgoing() && (config.getOutgoingTopicFilter() == null ||
                    config.getOutgoingTopicFilter()
                            .stream()
                            .anyMatch(tp -> MqttTopicFilter.of(tp)
                                    .matches(MqttTopic.of(messageDTO.getTopic()))
                            )
            )) {
                LOGGER.info(loggerUtils.getConnectionMarker(connectionId), "[HOOK] Manipulated outgoing message on {} with {}", messageDTO.getTopic(), p.getClass().getName());
                messageExtensionDTO = p.onPublishMessage(connectionId, messageExtensionDTO);
            }
        }
        return MessageExtensionTransformer.mergeDTO(messageExtensionDTO, messageDTO);
    }
}
