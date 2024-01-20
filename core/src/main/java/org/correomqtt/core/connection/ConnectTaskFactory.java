package org.correomqtt.core.connection;

import dagger.assisted.AssistedFactory;
import org.correomqtt.core.model.SubscriptionDTO;
import org.correomqtt.core.pubsub.SubscribeTask;

@AssistedFactory
public interface ConnectTaskFactory {
    ConnectTask create(String connectionId);
}