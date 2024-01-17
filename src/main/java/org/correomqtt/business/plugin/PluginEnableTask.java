package org.correomqtt.business.plugin;

import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.plugin.manager.PluginManager;

public class PluginEnableTask extends SimpleTask {

    private final String pluginId;

    public PluginEnableTask(String pluginId) {
        this.pluginId = pluginId;
    }

    @Override
    protected void execute() {
        PluginManager.getInstance().enablePlugin(pluginId);
        EventBus.fireAsync(new PluginEnabledEvent(pluginId));
    }

    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new PluginEnabledStartedEvent(pluginId));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new PluginEnabledFailedEvent(pluginId));
    }
}
