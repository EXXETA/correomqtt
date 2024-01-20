package org.correomqtt.core.plugin.marketplace;

import dagger.assisted.AssistedFactory;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.pubsub.PublishTask;

@AssistedFactory
public interface PluginEnableTaskFactory {
    PluginEnableTask create(String pluginId);
}