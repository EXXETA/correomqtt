package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.plugin.PluginManager;

@DefaultBean
public class PluginEnableTask extends SimpleTask {

    private final PluginManager pluginManager;
    private final SoyEvents soyEvents;
    private final String pluginId;

    @Inject
    public PluginEnableTask(PluginManager pluginManager,
                            SoyEvents soyEvents,
                            @Assisted String pluginId) {
        super(soyEvents);
        this.pluginManager = pluginManager;
        this.soyEvents = soyEvents;
        this.pluginId = pluginId;
    }

    @Override
    protected void execute() {
        pluginManager.enablePlugin(pluginId);
        soyEvents.fireAsync(new PluginEnabledEvent(pluginId));
    }

    @Override
    protected void beforeHook() {
        soyEvents.fireAsync(new PluginEnabledStartedEvent(pluginId));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        soyEvents.fireAsync(new PluginEnabledFailedEvent(pluginId));
    }
}
