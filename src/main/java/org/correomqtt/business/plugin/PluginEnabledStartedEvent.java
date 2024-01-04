package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginEnabledStartedEvent(String pluginId) implements Event {
}
