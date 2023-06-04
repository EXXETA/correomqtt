package org.correomqtt.business.dispatcher;

public interface PluginInstallObserver extends BaseObserver {
    void onPluginInstallSucceeded(String pluginId, String version);

    void onPluginInstallCancelled(String pluginId, String version);

    void onPluginInstallFailed(String pluginId, String version, Throwable exception);

    default void onPluginInstallRunning(String pluginId, String version) {
        // nothing to do
    }

    default void onPluginInstallScheduled(String pluginId, String version) {
        // nothing to do
    }

    default void onPluginInstallStarted(String pluginId, String version) {
        // nothing to do
    }
}
