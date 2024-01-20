package org.correomqtt.core.pubsub;

import dagger.assisted.AssistedFactory;
import org.correomqtt.core.model.MessageDTO;

@AssistedFactory
public interface PublishTaskFactory {
    PublishTask create(String connectionId, MessageDTO messageDT);
}