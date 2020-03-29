package org.correomqtt.business.dispatcher;

public class StartupDispatcher extends BaseDispatcher<StartupObserver> {

    private static StartupDispatcher instance;

    public static synchronized StartupDispatcher getInstance() {
        if (instance == null) {
            instance = new StartupDispatcher();
        }
        return instance;
    }

    public void onPluginUpdateFailed(String disabledPath) {
        trigger(o -> o.onPluginUpdateFailed(disabledPath));
    }

    public void onPluginLoadFailed() {
        trigger(StartupObserver::onPluginLoadFailed);
    }
}
