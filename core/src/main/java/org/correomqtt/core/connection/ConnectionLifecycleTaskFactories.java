package org.correomqtt.core.connection;

import lombok.Getter;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;

@Getter
@DefaultBean
public class ConnectionLifecycleTaskFactories {

    private final ConnectTaskFactory connectFactory;
    private final DisconnectTaskFactory disconnectFactory;
    private final ReconnectTaskFactory reconnectFactory;

    @Inject
    public ConnectionLifecycleTaskFactories(ConnectTaskFactory connectFactory,
                                            DisconnectTaskFactory disconnectFactory,
                                            ReconnectTaskFactory reconnectFactory) {
        this.connectFactory = connectFactory;
        this.disconnectFactory = disconnectFactory;
        this.reconnectFactory = reconnectFactory;
    }
}
