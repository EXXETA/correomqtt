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
    private final EventBus eventBus;
    private final String pluginId;
    private final String version;

    @AssistedFactory
    public interface Factory {
        PluginInstallTask create(@Assisted("pluginId") String pluginId, @Assisted("version") String version);
    }

    @AssistedInject
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
