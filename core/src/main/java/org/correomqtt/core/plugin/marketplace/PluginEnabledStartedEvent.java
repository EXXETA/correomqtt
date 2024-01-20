package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginEnabledStartedEvent(String pluginId) implements Event {
}
