package com.exxeta.correomqtt.business.dispatcher;

import com.exxeta.correomqtt.business.model.MessageDTO;

public interface PublishObserver extends BaseConnectionObserver {
    void onPublishSucceeded(MessageDTO messageDTO);

    void onPublishCancelled(MessageDTO messageDTO);

    void onPublishFailed(MessageDTO messageDTO, Throwable exception);

    default void onPublishRunning(MessageDTO messageDTO) {
        // nothing to do
    }

    default void onPublishScheduled(MessageDTO messageDTO) {
        // nothing to do
    }
}
