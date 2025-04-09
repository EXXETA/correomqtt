package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginEnabledFailedEvent(String pluginId) implements Event {
}
