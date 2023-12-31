package org.correomqtt.business.plugin;

import org.correomqtt.business.concurrent.Task;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.plugin.manager.PluginManager;

public class PluginDisableTask extends Task<Void,Void> {

    private final String pluginId;

    public PluginDisableTask(String pluginId){
        this.pluginId = pluginId;
    }
    @Override
    protected Void execute() throws Exception {
        PluginManager.getInstance().disablePlugin(pluginId);
        EventBus.fireAsync(new PluginDisabledEvent(pluginId));
        return null;
    }

    @Override
    protected void before() {
        EventBus.fireAsync(new PluginDisabledStartedEvent(pluginId));
    }

    @Override
    protected void error(Throwable throwable) {
        EventBus.fireAsync(new PluginDisabledFailedEvent(pluginId));
    }
}
