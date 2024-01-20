package org.correomqtt.core.pubsub;

import dagger.assisted.AssistedFactory;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.SubscriptionDTO;

@AssistedFactory
public interface SubscribeTaskFactory {
    SubscribeTask create(String connectionId, SubscriptionDTO subscriptionDTO);
}