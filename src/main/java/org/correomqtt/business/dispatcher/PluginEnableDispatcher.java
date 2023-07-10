package org.correomqtt.business.dispatcher;

public class PluginEnableDispatcher extends BaseDispatcher<PluginEnableObserver> {

    private static PluginEnableDispatcher instance;

    public static synchronized PluginEnableDispatcher getInstance() {
        if (instance == null) {
            instance = new PluginEnableDispatcher();
        }
        return instance;
    }

    public void onEnableSucceeded(String pluginId) {
        trigger(o -> o.onPluginEnableSucceeded(pluginId));
    }

    public void onEnableCancelled(String pluginId) {
        trigger(o -> o.onPluginEnableCancelled(pluginId));
    }

    public void onEnableFailed(String pluginId, Throwable exception) {
        trigger(o -> o.onPluginEnableFailed(pluginId, exception));
    }

    public void onEnableRunning(String pluginId) {
        trigger(o -> o.onPluginEnableRunning(pluginId));
    }

    public void onEnableScheduled(String pluginId) {
        trigger(o -> o.onPluginEnableScheduled(pluginId));
    }

    public void onEnableStarted(String pluginId) {
        trigger(o -> o.onPluginEnableStarted(pluginId));
    }
}
