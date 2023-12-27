package org.correomqtt.business.dispatcher;

public interface ShutdownObserver extends BaseObserver {
    void onShutdownRequested();
}
