package org.correomqtt.business.plugin;

import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.plugin.manager.PluginManager;

public class PluginDisableTask extends SimpleTask {

    private final String pluginId;

    public PluginDisableTask(String pluginId) {
        this.pluginId = pluginId;
    }

    @Override
    protected void execute() {
        PluginManager.getInstance().disablePlugin(pluginId);
        EventBus.fireAsync(new PluginDisabledEvent(pluginId));
    }

    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new PluginDisabledStartedEvent(pluginId));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new PluginDisabledFailedEvent(pluginId));
    }
}
