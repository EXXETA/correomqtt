package org.correomqtt.core.plugin.marketplace;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.plugin.PluginManager;

public class PluginUninstallTask extends SimpleTask {

    private final PluginManager pluginManager;
    private final String pluginId;

    @AssistedFactory
    public interface Factory {
        PluginUninstallTask create(String pluginId);
    }
    @AssistedInject
    public PluginUninstallTask(PluginManager pluginManager, @Assisted String pluginId) {
        this.pluginManager = pluginManager;
        this.pluginId = pluginId;
    }

    @Override
    protected void execute() {
        pluginManager.getUpdateManager().uninstallPlugin(pluginId);
        EventBus.fireAsync(new PluginUninstallEvent(pluginId));
    }

    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new PluginUninstallStartedEvent(pluginId));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult ignore) {
        EventBus.fireAsync(new PluginUninstallFailedEvent(pluginId));
    }
}
