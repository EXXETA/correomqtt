package org.correomqtt.business.services;

import com.hivemq.client.mqtt.datatypes.MqttTopic;
import com.hivemq.client.mqtt.datatypes.MqttTopicFilter;
import org.correomqtt.business.dispatcher.SubscribeDispatcher;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
import org.correomqtt.gui.transformer.MessageTransformer;
import org.correomqtt.plugin.manager.PluginManager;
import org.correomqtt.plugin.model.MessageExtensionDTO;
import org.correomqtt.plugin.spi.IncomingMessageHook;
import org.correomqtt.plugin.spi.IncomingMessageHookDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeoutException;

public class SubscribeService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(SubscribeService.class);

    private final SubscriptionDTO subscriptionDTO;

    public SubscribeService(String connectionId, SubscriptionDTO subscriptionDTO) {
        super(connectionId);
        this.subscriptionDTO = subscriptionDTO;
    }

    public void subscribe() {
        assert !subscriptionDTO.getTopic().isEmpty();
        callSafeOnClient(client -> subscribe(client, subscriptionDTO));
    }

    private void subscribe(CorreoMqttClient client, SubscriptionDTO subscriptionDTO) {

        try {
            client.subscribe(subscriptionDTO, (messageDTO -> {
                        MessageDTO manipulatedMessageDTO = executeOnMessageIncomingExtensions(messageDTO);
                        SubscribeDispatcher.getInstance().onMessageIncoming(connectionId, manipulatedMessageDTO, subscriptionDTO);
                    })
            );
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new CorreoMqttExecutionException(e);
        } catch (ExecutionException | TimeoutException e) {
            throw new CorreoMqttExecutionException(e);
        }
    }

    private MessageDTO executeOnMessageIncomingExtensions(MessageDTO messageDTO) {
        MessageExtensionDTO messageExtensionDTO = new MessageExtensionDTO(messageDTO);
        for (IncomingMessageHook<?> p : PluginManager.getInstance().getIncomingMessageHooks()) {
            IncomingMessageHookDTO config = p.getConfig();
            if (config != null && config.isEnableIncoming() && (config.getIncomingTopicFilter() == null ||
                    config.getIncomingTopicFilter()
                            .stream()
                            .anyMatch(tp -> MqttTopicFilter.of(tp)
                                            .matches(MqttTopic.of(messageDTO.getTopic()))
                            )
            )){
                LOGGER.info(getConnectionMarker(), "[HOOK] Manipulated incoming message on {} with {}", messageDTO.getTopic(), p.getClass().getName());
                messageExtensionDTO = p.onMessageIncoming(connectionId, messageExtensionDTO);
            }
        }
        return MessageTransformer.mergeDTO(messageExtensionDTO, messageDTO);
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Successful subscription to {}", subscriptionDTO.getTopic());
        SubscribeDispatcher.getInstance().onSubscribedSucceeded(connectionId, subscriptionDTO);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Subscription to {} cancelled", subscriptionDTO.getTopic());
        SubscribeDispatcher.getInstance().onSubscribedCanceled(connectionId, subscriptionDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info(getConnectionMarker(), "Subscription to {} failed", subscriptionDTO.getTopic(), exception);
        SubscribeDispatcher.getInstance().onSubscribedFailed(connectionId, subscriptionDTO, exception);

    }

    @Override
    public void onRunning() {
        LOGGER.debug(getConnectionMarker(), "Subscription to {} running.", subscriptionDTO.getTopic());
        SubscribeDispatcher.getInstance().onSubscribedRunning(connectionId, subscriptionDTO);
    }

    @Override
    public void onScheduled() {
        LOGGER.debug(getConnectionMarker(), "Subscription to {} scheduled.", subscriptionDTO.getTopic());
        SubscribeDispatcher.getInstance().onSubscribedScheduled(connectionId, subscriptionDTO);
    }
}
