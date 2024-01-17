package org.correomqtt.business.plugin;

import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.plugin.manager.PluginManager;

public class PluginInstallTask extends SimpleTask {

    private final String pluginId;
    private final String version;

    public PluginInstallTask(String pluginId, String version) {
        this.pluginId = pluginId;
        this.version = version;
    }

    @Override
    protected void execute() {
        PluginManager.getInstance().getUpdateManager().installPlugin(pluginId, version);
        EventBus.fireAsync(new PluginInstallEvent(pluginId, version));
    }

    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new PluginInstallStartedEvent(pluginId, version));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new PluginInstallFailedEvent(pluginId, version));
    }
}
