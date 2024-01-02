package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginEnabledEvent(String pluginId) implements Event {
}
