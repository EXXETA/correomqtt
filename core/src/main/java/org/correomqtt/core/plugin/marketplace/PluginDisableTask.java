package org.correomqtt.core.plugin.marketplace;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.plugin.PluginManager;

public class PluginDisableTask extends SimpleTask {

    private final PluginManager pluginManager;
    private final EventBus eventBus;
    private final String pluginId;

    @AssistedFactory
    public interface Factory {
        PluginDisableTask create(String pluginId);
    }

    @AssistedInject
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
