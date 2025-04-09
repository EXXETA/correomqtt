package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginEnabledStartedEvent(String pluginId) implements Event {
}
