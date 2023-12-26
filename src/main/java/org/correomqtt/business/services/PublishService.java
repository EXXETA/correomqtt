package org.correomqtt.business.services;

import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import org.correomqtt.business.dispatcher.PublishDispatcher;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.OutgoingMessageHook;
import org.correomqtt.plugin.spi.OutgoingMessageHookDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

public class PublishService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(PublishService.class);

    private final MessageDTO messageDTO;

    public PublishService(String connectionId, MessageDTO messageDTO) {
        super(connectionId);
        this.messageDTO = messageDTO;
    }

    public void publish() {
        LOGGER.info(getConnectionMarker(), "Start publishing to topic: {}", messageDTO.getTopic());
        callSafeOnClient(client -> {
            try {
                MessageDTO manipulatedMessageDTO = executeOnPublishMessageExtensions(connectionId, messageDTO);
                client.publish(manipulatedMessageDTO);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new CorreoMqttExecutionException(e);
            } catch (ExecutionException | TimeoutException e) {
                throw new CorreoMqttExecutionException(e);
            }
        });
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
            )){
                LOGGER.info(getConnectionMarker(), "[HOOK] Manipulated outgoing message on {} with {}", messageDTO.getTopic(), p.getClass().getName());
                messageExtensionDTO = p.onPublishMessage(connectionId, messageExtensionDTO);
            }
        }
        return MessageTransformer.mergeDTO(messageExtensionDTO, messageDTO);
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Publish to {} succeeded.", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishSucceeded(connectionId, messageDTO);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Publish to {} cancelled.", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishCancelled(connectionId, messageDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.warn(getConnectionMarker(), "Publish to {} failed: ", messageDTO.getTopic(), exception);
        PublishDispatcher.getInstance().onPublishFailed(connectionId, messageDTO, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.debug(getConnectionMarker(), "Publish to {} running", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishRunning(connectionId, messageDTO);
    }

    @Override
    public void onScheduled() {
        LOGGER.debug(getConnectionMarker(), "Publish to {} scheduled", messageDTO.getTopic());
        PublishDispatcher.getInstance().onPublishScheduled(connectionId, messageDTO);
    }
}
