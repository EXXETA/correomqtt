package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.PluginEnableDispatcher;
import org.correomqtt.plugin.manager.PluginManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class PluginEnableService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(PluginEnableService.class);

    private final String pluginId;

    public PluginEnableService(String pluginId) {
        this.pluginId = pluginId;
    }

    public void enable() {
        PluginEnableDispatcher.getInstance().onEnableStarted(pluginId);
        LOGGER.info("Start enabling plugin {}", pluginId);
        PluginManager.getInstance().enablePlugin(pluginId);

    }

    @Override
    public void onSucceeded() {
        LOGGER.info("Plugin enabled {}", pluginId);
        PluginEnableDispatcher.getInstance().onEnableSucceeded(pluginId);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Plugin enable cancelled {}", pluginId);
        PluginEnableDispatcher.getInstance().onEnableCancelled(pluginId);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Plugin enable failed {}", pluginId);
        PluginEnableDispatcher.getInstance().onEnableFailed(pluginId, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Plugin enable running {}", pluginId);
        PluginEnableDispatcher.getInstance().onEnableRunning(pluginId);
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Plugin enable scheduled {}", pluginId);
        PluginEnableDispatcher.getInstance().onEnableScheduled(pluginId);
    }
}

