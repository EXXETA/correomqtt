package org.correomqtt.core.connection;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface DisconnectTaskFactory {
    DisconnectTask create(String connectionId);
}