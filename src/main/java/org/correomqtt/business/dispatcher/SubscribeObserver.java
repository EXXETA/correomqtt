package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.SubscriptionDTO;

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
