package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginDisabledFailedEvent(String pluginId) implements Event {
}
