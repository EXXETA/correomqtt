package org.correomqtt.core.connection;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ReconnectTaskFactory {
    ReconnectTask create(String connectionId);
}