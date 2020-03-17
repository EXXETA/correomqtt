package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.UnsubscribeDispatcher;
import org.correomqtt.business.model.SubscriptionDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class UnsubscribeService extends BaseService {

    private static final Logger LOGGER = LoggerFactory.getLogger(UnsubscribeService.class);

    private final SubscriptionDTO subscriptionDTO;

    public UnsubscribeService(String connectionId, SubscriptionDTO subscriptionDTO) {
        super(connectionId);
        this.subscriptionDTO = subscriptionDTO;
    }

    public void unsubscribe() {
        LOGGER.info(getConnectionMarker(), "Start unsubscribing to topic: {}", subscriptionDTO.getTopic());
        callSafeOnClient(client -> client.unsubscribe(subscriptionDTO));
    }

    @Override
    public void onSucceeded() {
        LOGGER.info(getConnectionMarker(), "Successfully unsubscribed from {}.", subscriptionDTO.getTopic());
        UnsubscribeDispatcher.getInstance().onUnsubscribeSucceeded(connectionId, subscriptionDTO);
    }

    @Override
    public void onCancelled() {
        LOGGER.info(getConnectionMarker(), "Unsubscribe from {} cancelled.", subscriptionDTO.getTopic());
        UnsubscribeDispatcher.getInstance().onUnsubscribeCanceled(connectionId, subscriptionDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.warn(getConnectionMarker(), "Unsubscribe from {} failed: ", subscriptionDTO.getTopic(), exception);
        UnsubscribeDispatcher.getInstance().onUnsubscribeFailed(connectionId, subscriptionDTO, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.debug(getConnectionMarker(), "Unsubscribe from {} running.", subscriptionDTO.getTopic());
        UnsubscribeDispatcher.getInstance().onUnsubscribeRunning(connectionId, subscriptionDTO);
    }

    @Override
    public void onScheduled() {
        LOGGER.debug(getConnectionMarker(), "Unsubscribe from {} scheduled.", subscriptionDTO.getTopic());
        UnsubscribeDispatcher.getInstance().onUnsubscribeScheduled(connectionId, subscriptionDTO);
    }
}
