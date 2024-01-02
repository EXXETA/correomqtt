package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginDisabledFailedEvent(String pluginId) implements Event {
}
