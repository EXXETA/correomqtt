package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.PluginUninstallDispatcher;
import org.correomqtt.plugin.manager.PluginManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class PluginUninstallService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginUninstallService.class);

    private final String pluginId;
    
    public PluginUninstallService(String pluginId) {
        this.pluginId = pluginId;
    }

    public void uninstall() {
        PluginUninstallDispatcher.getInstance().onUninstallStarted(pluginId);
        LOGGER.info("Start Uninstalling plugin {}", pluginId);
        PluginManager.getInstance().getUpdateManager().uninstallPlugin(pluginId);

    }

    @Override
    public void onSucceeded() {
        LOGGER.info("Plugin Uninstalled {}", pluginId);
        PluginUninstallDispatcher.getInstance().onUninstallSucceeded(pluginId);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Plugin Uninstallation cancelled {}", pluginId);
        PluginUninstallDispatcher.getInstance().onUninstallCancelled(pluginId);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Plugin Uninstallation failed {}", pluginId);
        PluginUninstallDispatcher.getInstance().onUninstallFailed(pluginId, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Plugin Uninstallation running {}", pluginId);
        PluginUninstallDispatcher.getInstance().onUninstallRunning(pluginId);
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Plugin Uninstallation scheduled {}", pluginId);
        PluginUninstallDispatcher.getInstance().onUninstallScheduled(pluginId);
    }
}

