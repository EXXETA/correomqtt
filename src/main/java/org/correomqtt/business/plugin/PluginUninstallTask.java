package org.correomqtt.business.plugin;

import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.plugin.manager.PluginManager;

public class PluginUninstallTask extends SimpleTask {

    private final String pluginId;

    public PluginUninstallTask(String pluginId) {
        this.pluginId = pluginId;
    }

    @Override
    protected void execute() throws Exception {
        PluginManager.getInstance().getUpdateManager().uninstallPlugin(pluginId);
        EventBus.fireAsync(new PluginUninstallEvent(pluginId));
    }

    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new PluginUninstallStartedEvent(pluginId));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new PluginUninstallFailedEvent(pluginId));
    }
}
