package org.correomqtt.core.plugin.marketplace;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.plugin.PluginManager;

public class PluginInstallTask extends SimpleTask {

    private final PluginManager pluginManager;
    private final String pluginId;
    private final String version;

    @AssistedFactory
    public interface Factory {
        PluginInstallTask create(@Assisted("pluginId") String pluginId, @Assisted("version") String version);
    }
    @AssistedInject
    public PluginInstallTask(PluginManager pluginManager,
                             @Assisted("pluginId") String pluginId,
                             @Assisted("version") String version) {
        this.pluginManager = pluginManager;
        this.pluginId = pluginId;
        this.version = version;
    }

    @Override
    protected void execute() {
        pluginManager.getUpdateManager().installPlugin(pluginId, version);
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
