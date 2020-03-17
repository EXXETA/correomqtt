package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.SubscriptionDTO;

public class SubscribeDispatcher extends BaseConnectionDispatcher<SubscribeObserver> {

    private static SubscribeDispatcher instance;

    public static synchronized SubscribeDispatcher getInstance() {
        if (instance == null) {
            instance = new SubscribeDispatcher();
        }
        return instance;
    }

    public void onMessageIncoming(String connectionId, MessageDTO messageDTO, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onMessageIncoming(messageDTO, subscriptionDTO));
    }

    public void onSubscribedSucceeded(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onSubscribedSucceeded(subscriptionDTO));
        SubscribeGlobalDispatcher.getInstance().onSubscribedSucceeded(connectionId, subscriptionDTO);
    }

    public void onSubscribedCanceled(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onSubscribedCanceled(subscriptionDTO));
    }

    public void onSubscribedFailed(String connectionId, SubscriptionDTO subscriptionDTO, Throwable exception) {
        triggerFiltered(connectionId, o -> o.onSubscribedFailed(subscriptionDTO, exception));
    }

    public void onSubscribedRunning(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onSubscribedRunning(subscriptionDTO));
    }

    public void onSubscribedScheduled(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onSubscribedScheduled(subscriptionDTO));
    }
}
