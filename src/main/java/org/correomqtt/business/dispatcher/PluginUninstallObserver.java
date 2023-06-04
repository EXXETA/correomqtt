package org.correomqtt.business.dispatcher;

public interface PluginUninstallObserver extends BaseObserver {
    void onPluginUninstallSucceeded(String pluginId);

    void onPluginUninstallCancelled(String pluginId);

    void onPluginUninstallFailed(String pluginId, Throwable exception);

    default void onPluginUninstallRunning(String pluginId) {
        // nothing to do
    }

    default void onPluginUninstallScheduled(String pluginId) {
        // nothing to do
    }

    default void onPluginUninstallStarted(String pluginId) {
        // nothing to do
    }
}
