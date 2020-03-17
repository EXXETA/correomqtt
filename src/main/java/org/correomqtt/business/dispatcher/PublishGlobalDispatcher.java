package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.MessageDTO;

public class PublishGlobalDispatcher extends BaseConnectionDispatcher<PublishGlobalObserver> {

    private static PublishGlobalDispatcher instance;

    public static synchronized PublishGlobalDispatcher getInstance() {
        if (instance == null) {
            instance = new PublishGlobalDispatcher();
        }
        return instance;
    }

    public void onPublishSuceeded(String connectionId, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onPublishSucceeded(connectionId, messageDTO));
    }

    public void onPublishRemoved(String connectionId, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onPublishRemoved(connectionId, messageDTO));
    }

    public void onPublishesCleared(String connectionId){
        triggerFiltered(connectionId, o -> o.onPublishesCleared(connectionId));
    }

}
