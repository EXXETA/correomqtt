package org.correomqtt.core.plugin.marketplace;

import dagger.assisted.AssistedFactory;
import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.pubsub.PublishTask;

@AssistedFactory
public interface PluginDisableTaskFactory {
    PluginDisableTask create(String pluginId);
}