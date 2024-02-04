package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginDisabledStartedEvent(String pluginId) implements Event {
}
