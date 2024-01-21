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
    private final String pluginId;
    @AssistedFactory
    public interface Factory {
        PluginDisableTask create(String pluginId);
    }
    @AssistedInject
    public PluginDisableTask(PluginManager pluginManager, @Assisted String pluginId) {
        this.pluginManager = pluginManager;
        this.pluginId = pluginId;
    }

    @Override
    protected void execute() {
        pluginManager.disablePlugin(pluginId);
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
