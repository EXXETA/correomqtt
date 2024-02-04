package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginDisabledStartedEvent(String pluginId) implements Event {
}
