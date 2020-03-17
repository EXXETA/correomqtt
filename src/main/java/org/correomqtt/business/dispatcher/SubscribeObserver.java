package com.exxeta.correomqtt.business.dispatcher;

import com.exxeta.correomqtt.business.model.MessageDTO;
import com.exxeta.correomqtt.business.model.SubscriptionDTO;

public interface SubscribeObserver extends BaseConnectionObserver {
    void onMessageIncoming(MessageDTO messageDTO, SubscriptionDTO subscriptionDTO);

    void onSubscribedSucceeded(SubscriptionDTO subscriptionDTO);

    void onSubscribedCanceled(SubscriptionDTO subscriptionDTO);

    void onSubscribedFailed(SubscriptionDTO subscriptionDTO, Throwable exception);

    default void onSubscribedRunning(SubscriptionDTO subscriptionDTO) {
        // nothing to do
    }

    default void onSubscribedScheduled(SubscriptionDTO subscriptionDTO) {
        // nothing to do
    }
}
