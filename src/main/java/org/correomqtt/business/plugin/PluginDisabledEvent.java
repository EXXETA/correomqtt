package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginDisabledEvent(String pluginId) implements Event {
}
