package org.correomqtt.business.plugin;

import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.plugin.manager.PluginManager;

public class PluginUninstallTask extends Task<Void, Void> {

    private final String pluginId;

    public PluginUninstallTask(String pluginId) {
        this.pluginId = pluginId;
    }

    @Override
    protected Void execute() throws Exception {
        PluginManager.getInstance().getUpdateManager().uninstallPlugin(pluginId);
        EventBus.fireAsync(new PluginUninstallEvent(pluginId));
        return null;
    }

    @Override
    protected void before() {
        EventBus.fireAsync(new PluginUninstallStartedEvent(pluginId));
    }

    @Override
    protected void error(Throwable throwable) {
        EventBus.fireAsync(new PluginUninstallFailedEvent(pluginId));
    }
}
