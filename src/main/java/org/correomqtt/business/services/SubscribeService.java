package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.SubscribeDispatcher;
import org.correomqtt.business.exception.CorreoMqttExecutionException;
import org.correomqtt.business.model.SubscriptionDTO;
import org.correomqtt.business.mqtt.CorreoMqttClient;
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
            client.subscribe(subscriptionDTO, (messageDTO ->
                    SubscribeDispatcher.getInstance().onMessageIncoming(connectionId, messageDTO, subscriptionDTO))
            );
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new CorreoMqttExecutionException(e);
        } catch (ExecutionException | TimeoutException e) {
            throw new CorreoMqttExecutionException(e);
        }

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
