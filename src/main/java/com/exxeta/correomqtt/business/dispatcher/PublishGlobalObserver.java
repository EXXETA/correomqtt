package com.exxeta.correomqtt.business.dispatcher;

import com.exxeta.correomqtt.business.model.MessageDTO;

public interface PublishGlobalObserver extends BaseConnectionObserver {

    void onPublishSucceeded(String connectionId, MessageDTO messageDTO);
    void onPublishRemoved(String connectionId, MessageDTO messageDTO);
    void onPublishesCleared(String connectionId);
}
