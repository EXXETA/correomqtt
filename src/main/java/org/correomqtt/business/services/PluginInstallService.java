package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.PluginInstallDispatcher;
import org.correomqtt.plugin.manager.PluginManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class PluginInstallService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginInstallService.class);

    private final String pluginId;
    private final String version;

    public PluginInstallService(String pluginId, String version) {
        this.pluginId = pluginId;
        this.version = version;
    }

    public void install() {
        PluginInstallDispatcher.getInstance().onInstallStarted(pluginId, version);
        LOGGER.info("Start installing plugin {}@{}", pluginId, version);
        PluginManager.getInstance().getUpdateManager().installPlugin(pluginId, version);
    }

    @Override
    public void onSucceeded() {
        LOGGER.info("Plugin installed {}@{}", pluginId, version);
        PluginInstallDispatcher.getInstance().onInstallSucceeded(pluginId, version);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Plugin installation cancelled {}@{}", pluginId, version);
        PluginInstallDispatcher.getInstance().onInstallCancelled(pluginId, version);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Plugin installation failed {}@{}", pluginId, version, exception);
        PluginInstallDispatcher.getInstance().onInstallFailed(pluginId, version, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Plugin installation running {}@{}", pluginId, version);
        PluginInstallDispatcher.getInstance().onInstallRunning(pluginId, version);
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Plugin installation scheduled {}@{}", pluginId, version);
        PluginInstallDispatcher.getInstance().onInstallScheduled(pluginId, version);
    }
}

