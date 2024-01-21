package org.correomqtt.core.fileprovider;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface PublishMessageHistoryFactory {
    PublishMessageHistory create(String connectionId);

}
