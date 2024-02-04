package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.plugin.PluginManager;

@DefaultBean
public class PluginDisableTask extends SimpleTask {

    private final PluginManager pluginManager;
    private final EventBus eventBus;
    private final String pluginId;



    @Inject
    public PluginDisableTask(PluginManager pluginManager,
                             EventBus eventBus,
                             @Assisted String pluginId) {
        super(eventBus);
        this.pluginManager = pluginManager;
        this.eventBus = eventBus;
        this.pluginId = pluginId;
    }

    @Override
    protected void execute() {
        pluginManager.disablePlugin(pluginId);
        eventBus.fireAsync(new PluginDisabledEvent(pluginId));
    }

    @Override
    protected void beforeHook() {
        eventBus.fireAsync(new PluginDisabledStartedEvent(pluginId));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        eventBus.fireAsync(new PluginDisabledFailedEvent(pluginId));
    }
}
