package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.SubscriptionDTO;

public interface UnsubscribeObserver extends BaseConnectionObserver {

    void onUnsubscribeSucceeded(SubscriptionDTO subscriptionDTO);

    void onUnsubscribeCanceled(SubscriptionDTO subscriptionDTO);

    void onUnsubscribeFailed(SubscriptionDTO subscriptionDTO, Throwable exception);

    default void onUnsubscribeRunning(SubscriptionDTO subscriptionDTO) {
        // nothing to do
    }

    default void onUnsubscribeScheduled(SubscriptionDTO subscriptionDTO) {
        // nothing to do
    }
}
