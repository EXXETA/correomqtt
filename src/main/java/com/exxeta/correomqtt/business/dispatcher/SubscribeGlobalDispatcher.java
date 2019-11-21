package com.exxeta.correomqtt.business.dispatcher;

import com.exxeta.correomqtt.business.model.SubscriptionDTO;

public class SubscribeGlobalDispatcher extends BaseConnectionDispatcher<SubscribeGlobalObserver> {

    private static SubscribeGlobalDispatcher instance;

    public static synchronized SubscribeGlobalDispatcher getInstance() {
        if (instance == null) {
            instance = new SubscribeGlobalDispatcher();
        }
        return instance;
    }

    public void onSubscribedSucceeded(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onSubscribedSucceeded(connectionId, subscriptionDTO));
    }

    public void onSubscribeRemoved(String connectionId, SubscriptionDTO subscriptionDTO) {
        triggerFiltered(connectionId, o -> o.onSubscribeRemoved(connectionId, subscriptionDTO));
    }

    public void onSubscribeCleared(String connectionId){
        triggerFiltered(connectionId, o -> o.onSubscribeCleared(connectionId));
    }

}
