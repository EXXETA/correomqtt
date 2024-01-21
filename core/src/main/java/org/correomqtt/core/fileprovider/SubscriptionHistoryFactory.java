package org.correomqtt.core.fileprovider;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface SubscriptionHistoryFactory {
    SubscriptionHistory create(String connectionId);

}
