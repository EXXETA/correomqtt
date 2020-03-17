package com.exxeta.correomqtt.business.dispatcher;

public class ApplicationLifecycleDispatcher extends BaseDispatcher<ApplicationLifecycleObserver> {

    private static ApplicationLifecycleDispatcher instance;

    public static synchronized ApplicationLifecycleDispatcher getInstance() {
        if (instance == null) {
            instance = new ApplicationLifecycleDispatcher();
        }
        return instance;
    }

    public void onShutdown() {
        trigger(ApplicationLifecycleObserver::onShutdown);
    }

}
