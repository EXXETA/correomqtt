package org.correomqtt.core.pubsub;

import dagger.assisted.AssistedFactory;
import org.correomqtt.core.model.SubscriptionDTO;

@AssistedFactory
public interface UnsubscribeTaskFactory {
    UnsubscribeTask create(String connectionId, SubscriptionDTO subscriptionDTO);
}