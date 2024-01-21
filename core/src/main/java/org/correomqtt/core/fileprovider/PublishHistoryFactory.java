package org.correomqtt.core.fileprovider;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface PublishHistoryFactory {
    PublishHistory create(String connectionId);

}
