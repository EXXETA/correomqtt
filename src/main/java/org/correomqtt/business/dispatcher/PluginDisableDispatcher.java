package org.correomqtt.business.dispatcher;

public class PluginDisableDispatcher extends BaseDispatcher<PluginDisableObserver> {

    private static PluginDisableDispatcher instance;

    public static synchronized PluginDisableDispatcher getInstance() {
        if (instance == null) {
            instance = new PluginDisableDispatcher();
        }
        return instance;
    }

    public void onDisableSucceeded(String pluginId) {
        trigger(o -> o.onPluginDisableSucceeded(pluginId));
    }

    public void onDisableCancelled(String pluginId) {
        trigger(o -> o.onPluginDisableCancelled(pluginId));
    }

    public void onDisableFailed(String pluginId, Throwable exception) {
        trigger(o -> o.onPluginDisableFailed(pluginId, exception));
    }

    public void onDisableRunning(String pluginId) {
        trigger(o -> o.onPluginDisableRunning(pluginId));
    }

    public void onDisableScheduled(String pluginId) {
        trigger(o -> o.onPluginDisableScheduled(pluginId));
    }

    public void onDisableStarted(String pluginId) {
        trigger(o -> o.onPluginDisableStarted(pluginId));
    }
}
