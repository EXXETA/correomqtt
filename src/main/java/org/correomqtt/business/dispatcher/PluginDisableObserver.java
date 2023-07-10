package org.correomqtt.business.dispatcher;

public interface PluginDisableObserver extends BaseObserver {
    void onPluginDisableSucceeded(String pluginId);

    void onPluginDisableCancelled(String pluginId);

    void onPluginDisableFailed(String pluginId, Throwable exception);

    default void onPluginDisableRunning(String pluginId) {
        // nothing to do
    }

    default void onPluginDisableScheduled(String pluginId) {
        // nothing to do
    }

    default void onPluginDisableStarted(String pluginId) {
        // nothing to do
    }
}
