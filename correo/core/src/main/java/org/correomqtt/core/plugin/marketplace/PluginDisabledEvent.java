package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginDisabledEvent(String pluginId) implements Event {
}
