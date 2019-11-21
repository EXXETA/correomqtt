package com.exxeta.correomqtt.business.dispatcher;

public interface ApplicationLifecycleObserver extends BaseObserver {
    void onShutdown();
}
