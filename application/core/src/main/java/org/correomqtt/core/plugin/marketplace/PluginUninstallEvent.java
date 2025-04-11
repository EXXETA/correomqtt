package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginUninstallEvent(String pluginId) implements Event {
}
