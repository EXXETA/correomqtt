package org.correomqtt.core.plugin.marketplace;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.pubsub.PublishTask;

@AssistedFactory
public interface PluginInstallTaskFactory {
    PluginInstallTask create(@Assisted("pluginId") String pluginId, @Assisted("version") String version);
}