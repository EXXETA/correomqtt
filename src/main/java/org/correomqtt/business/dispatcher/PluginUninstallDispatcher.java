package org.correomqtt.business.dispatcher;

public class PluginUninstallDispatcher extends BaseDispatcher<PluginUninstallObserver> {

    private static PluginUninstallDispatcher instance;

    public static synchronized PluginUninstallDispatcher getInstance() {
        if (instance == null) {
            instance = new PluginUninstallDispatcher();
        }
        return instance;
    }

    public void onUninstallSucceeded(String pluginId) {
        trigger(o -> o.onPluginUninstallSucceeded(pluginId));
    }

    public void onUninstallCancelled(String pluginId) {
        trigger(o -> o.onPluginUninstallCancelled(pluginId));
    }

    public void onUninstallFailed(String pluginId, Throwable exception) {
        trigger(o -> o.onPluginUninstallFailed(pluginId, exception));
    }

    public void onUninstallRunning(String pluginId) {
        trigger(o -> o.onPluginUninstallRunning(pluginId));
    }

    public void onUninstallScheduled(String pluginId) {
        trigger(o -> o.onPluginUninstallScheduled(pluginId));
    }

    public void onUninstallStarted(String pluginId) {
        trigger(o -> o.onPluginUninstallStarted(pluginId));
    }
}
