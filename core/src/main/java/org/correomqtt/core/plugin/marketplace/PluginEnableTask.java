package org.correomqtt.core.plugin.marketplace;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.plugin.PluginManager;

public class PluginEnableTask extends SimpleTask {

    private final PluginManager pluginManager;
    private final String pluginId;

    @AssistedFactory
    public interface Factory {
        PluginEnableTask create(String pluginId);
    }

    @AssistedInject
    public PluginEnableTask(PluginManager pluginManager, @Assisted String pluginId) {
        this.pluginManager = pluginManager;
        this.pluginId = pluginId;
    }

    @Override
    protected void execute() {
        pluginManager.enablePlugin(pluginId);
        EventBus.fireAsync(new PluginEnabledEvent(pluginId));
    }

    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new PluginEnabledStartedEvent(pluginId));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new PluginEnabledFailedEvent(pluginId));
    }
}
