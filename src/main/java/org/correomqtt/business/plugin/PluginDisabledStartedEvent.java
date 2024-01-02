package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginDisabledStartedEvent(String pluginId) implements Event {
}
