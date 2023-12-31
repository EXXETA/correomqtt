package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginEnabledFailedEvent(String pluginId) implements Event {
}
