package org.correomqtt.business.dispatcher;

public class PluginInstallDispatcher extends BaseDispatcher<PluginInstallObserver> {

    private static PluginInstallDispatcher instance;

    public static synchronized PluginInstallDispatcher getInstance() {
        if (instance == null) {
            instance = new PluginInstallDispatcher();
        }
        return instance;
    }

    public void onInstallSucceeded(String pluginId, String version) {
        trigger(o -> o.onPluginInstallSucceeded(pluginId, version));
    }

    public void onInstallCancelled(String pluginId, String version) {
        trigger(o -> o.onPluginInstallCancelled(pluginId, version));
    }

    public void onInstallFailed(String pluginId, String version, Throwable exception) {
        trigger(o -> o.onPluginInstallFailed(pluginId, version, exception));
    }

    public void onInstallRunning(String pluginId, String version) {
        trigger(o -> o.onPluginInstallRunning(pluginId, version));
    }

    public void onInstallScheduled(String pluginId, String version) {
        trigger(o -> o.onPluginInstallScheduled(pluginId, version));
    }

    public void onInstallStarted(String pluginId, String version) {
        trigger(o -> o.onPluginInstallStarted(pluginId, version));
    }
}
