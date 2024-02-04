package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.plugin.PluginManager;

@DefaultBean
public class PluginInstallTask extends SimpleTask {

    private final PluginManager pluginManager;
    private final EventBus eventBus;
    private final String pluginId;
    private final String version;



    @Inject
    public PluginInstallTask(PluginManager pluginManager,
                             EventBus eventBus,
                             @Assisted("pluginId") String pluginId,
                             @Assisted("version") String version) {
        super(eventBus);
        this.pluginManager = pluginManager;
        this.eventBus = eventBus;
        this.pluginId = pluginId;
        this.version = version;
    }

    @Override
    protected void execute() {
        pluginManager.getUpdateManager().installPlugin(pluginId, version);
        eventBus.fireAsync(new PluginInstallEvent(pluginId, version));
    }

    @Override
    protected void beforeHook() {
        eventBus.fireAsync(new PluginInstallStartedEvent(pluginId, version));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        eventBus.fireAsync(new PluginInstallFailedEvent(pluginId, version));
    }
}
