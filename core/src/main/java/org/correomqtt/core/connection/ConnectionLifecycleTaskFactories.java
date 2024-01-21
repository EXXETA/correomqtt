package org.correomqtt.core.connection;

import lombok.Getter;

import javax.inject.Inject;

@Getter
public class ConnectionLifecycleTaskFactories {

    private final ConnectTask.Factory connectFactory;
    private final DisconnectTask.Factory disconnectFactory;
    private final ReconnectTask.Factory reconnectFactory;

    @Inject
    public ConnectionLifecycleTaskFactories(ConnectTask.Factory connectFactory,
                                            DisconnectTask.Factory disconnectFactory,
                                            ReconnectTask.Factory reconnectFactory) {
        this.connectFactory = connectFactory;
        this.disconnectFactory = disconnectFactory;
        this.reconnectFactory = reconnectFactory;
    }
}
