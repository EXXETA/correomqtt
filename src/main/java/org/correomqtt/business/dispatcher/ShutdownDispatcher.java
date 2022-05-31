package org.correomqtt.business.dispatcher;

public class ShutdownDispatcher extends BaseDispatcher<ShutdownObserver> {

    private static ShutdownDispatcher instance;

    public static synchronized ShutdownDispatcher getInstance() {
        if (instance == null) {
            instance = new ShutdownDispatcher();
        }
        return instance;
    }

    public void onShutdownRequested() {
        trigger(ShutdownObserver::onShutdownRequested);
    }
}
