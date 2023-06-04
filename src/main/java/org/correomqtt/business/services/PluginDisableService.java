package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.PluginDisableDispatcher;
import org.correomqtt.plugin.manager.PluginManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class PluginDisableService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginDisableService.class);

    private final String pluginId;

    public PluginDisableService(String pluginId) {
        this.pluginId = pluginId;
    }

    public void disable() {
        PluginDisableDispatcher.getInstance().onDisableStarted(pluginId);
        LOGGER.info("Start installing plugin {}", pluginId);
        PluginManager.getInstance().disablePlugin(pluginId);

    }

    @Override
    public void onSucceeded() {
        LOGGER.info("Plugin installed {}", pluginId);
        PluginDisableDispatcher.getInstance().onDisableSucceeded(pluginId);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Plugin installation cancelled {}", pluginId);
        PluginDisableDispatcher.getInstance().onDisableCancelled(pluginId);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Plugin installation failed {}", pluginId);
        PluginDisableDispatcher.getInstance().onDisableFailed(pluginId, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Plugin installation running {}", pluginId);
        PluginDisableDispatcher.getInstance().onDisableRunning(pluginId);
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Plugin installation scheduled {}", pluginId);
        PluginDisableDispatcher.getInstance().onDisableScheduled(pluginId);
    }
}

