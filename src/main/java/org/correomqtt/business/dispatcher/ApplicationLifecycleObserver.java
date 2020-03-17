package org.correomqtt.business.dispatcher;

public interface ApplicationLifecycleObserver extends BaseObserver {
    void onShutdown();
}
