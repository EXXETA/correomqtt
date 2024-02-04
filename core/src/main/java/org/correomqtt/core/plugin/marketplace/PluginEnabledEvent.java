package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginEnabledEvent(String pluginId) implements Event {
}
