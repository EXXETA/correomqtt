package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginDisabledEvent(String pluginId) implements Event {
}
