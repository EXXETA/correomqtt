package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.MessageDTO;

public interface PublishGlobalObserver extends BaseConnectionObserver {

    void onPublishSucceeded(String connectionId, MessageDTO messageDTO);
    void onPublishRemoved(String connectionId, MessageDTO messageDTO);
    void onPublishesCleared(String connectionId);
    void onPublishChangeFavoriteStatus(String connectionId, MessageDTO messageDTO);
}
