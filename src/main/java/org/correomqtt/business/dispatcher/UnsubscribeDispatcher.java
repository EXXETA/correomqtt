package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.SubscriptionDTO;

public class UnsubscribeDispatcher extends BaseConnectionDispatcher<UnsubscribeObserver> {

    private static UnsubscribeDispatcher instance;

    public static synchronized UnsubscribeDispatcher getInstance() {
        if (instance == null) {
            instance = new UnsubscribeDispatcher();
        }
        return instance;
    }

    public void onUnsubscribeSucceeded(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onUnsubscribeSucceeded(subscriptionDTO));
    }

    public void onUnsubscribeCanceled(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onUnsubscribeCanceled(subscriptionDTO));
    }

    public void onUnsubscribeFailed(String connectionId, SubscriptionDTO subscriptionDTO, Throwable exception) {
        triggerFiltered(connectionId, o -> o.onUnsubscribeFailed(subscriptionDTO, exception));
    }

    public void onUnsubscribeRunning(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onUnsubscribeRunning(subscriptionDTO));
    }

    public void onUnsubscribeScheduled(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onUnsubscribeScheduled(subscriptionDTO));
    }
}
