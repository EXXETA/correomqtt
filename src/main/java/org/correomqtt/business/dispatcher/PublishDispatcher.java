package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.MessageDTO;

public class PublishDispatcher extends BaseConnectionDispatcher<PublishObserver> {

    private static PublishDispatcher instance;

    public static synchronized PublishDispatcher getInstance() {
        if (instance == null) {
            instance = new PublishDispatcher();
        }
        return instance;
    }

    public void onPublishSucceeded(String connectionId, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onPublishSucceeded(messageDTO));
        PublishGlobalDispatcher.getInstance().onPublishSuceeded(connectionId,messageDTO);
    }

    public void onPublishCancelled(String connectionId, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onPublishCancelled(messageDTO));
    }

    public void onPublishFailed(String connectionId, MessageDTO messageDTO, Throwable exception) {
        triggerFiltered(connectionId, o -> o.onPublishFailed(messageDTO, exception));
    }

    public void onPublishRunning(String connectionId, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onPublishRunning(messageDTO));
    }

    public void onPublishScheduled(String connectionId, MessageDTO messageDTO) {
        triggerFiltered(connectionId, o -> o.onPublishScheduled(messageDTO));
    }
}
