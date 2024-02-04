package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginEnabledEvent(String pluginId) implements Event {
}
