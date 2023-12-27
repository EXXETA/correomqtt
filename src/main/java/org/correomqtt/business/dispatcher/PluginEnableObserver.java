package org.correomqtt.business.dispatcher;

public interface PluginEnableObserver extends BaseObserver {
    void onPluginEnableSucceeded(String pluginId);

    void onPluginEnableCancelled(String pluginId);

    void onPluginEnableFailed(String pluginId, Throwable exception);

    default void onPluginEnableRunning(String pluginId) {
        // nothing to do
    }

    default void onPluginEnableScheduled(String pluginId) {
        // nothing to do
    }

    default void onPluginEnableStarted(String pluginId) {
        // nothing to do
    }
}
