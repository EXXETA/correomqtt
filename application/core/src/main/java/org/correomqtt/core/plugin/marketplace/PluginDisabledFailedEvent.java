package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginDisabledFailedEvent(String pluginId) implements Event {
}
