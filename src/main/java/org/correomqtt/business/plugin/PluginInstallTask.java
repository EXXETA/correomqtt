package org.correomqtt.business.plugin;

import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.plugin.manager.PluginManager;

public class PluginInstallTask extends NoProgressTask<Void, Void> {

    private final String pluginId;
    private final String version;

    public PluginInstallTask(String pluginId, String version) {
        this.pluginId = pluginId;
        this.version = version;
    }

    @Override
    protected Void execute() throws Exception {
        PluginManager.getInstance().getUpdateManager().installPlugin(pluginId,version);
        EventBus.fireAsync(new PluginInstallEvent(pluginId,version));
        return null;
    }

    @Override
    protected void before() {
        EventBus.fireAsync(new PluginInstallStartedEvent(pluginId, version));
    }

    @Override
    protected void error(Throwable throwable) {
        EventBus.fireAsync(new PluginInstallFailedEvent(pluginId, version));
    }
}
