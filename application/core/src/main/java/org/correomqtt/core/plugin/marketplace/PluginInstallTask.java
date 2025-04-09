package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.plugin.PluginManager;

@DefaultBean
public class PluginInstallTask extends SimpleTask {

    private final PluginManager pluginManager;
    private final SoyEvents soyEvents;
    private final String pluginId;
    private final String version;



    @Inject
    public PluginInstallTask(PluginManager pluginManager,
                             SoyEvents soyEvents,
                             @Assisted("pluginId") String pluginId,
                             @Assisted("version") String version) {
        super(soyEvents);
        this.pluginManager = pluginManager;
        this.soyEvents = soyEvents;
        this.pluginId = pluginId;
        this.version = version;
    }

    @Override
    protected void execute() {
        pluginManager.getUpdateManager().installPlugin(pluginId, version);
        soyEvents.fireAsync(new PluginInstallEvent(pluginId, version));
    }

    @Override
    protected void beforeHook() {
        soyEvents.fireAsync(new PluginInstallStartedEvent(pluginId, version));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        soyEvents.fireAsync(new PluginInstallFailedEvent(pluginId, version));
    }
}
