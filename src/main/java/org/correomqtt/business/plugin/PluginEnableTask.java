package org.correomqtt.business.plugin;

import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.plugin.manager.PluginManager;

public class PluginEnableTask extends NoProgressTask<Void, Void> {

    private final String pluginId;

    public PluginEnableTask(String pluginId) {
        this.pluginId = pluginId;
    }

    @Override
    protected Void execute() throws Exception {
        PluginManager.getInstance().enablePlugin(pluginId);
        EventBus.fireAsync(new PluginEnabledEvent(pluginId));
        return null;
    }

    @Override
    protected void before() {
        EventBus.fireAsync(new PluginEnabledStartedEvent(pluginId));
    }

    @Override
    protected void error(Throwable throwable) {
        EventBus.fireAsync(new PluginEnabledFailedEvent(pluginId));
    }
}
