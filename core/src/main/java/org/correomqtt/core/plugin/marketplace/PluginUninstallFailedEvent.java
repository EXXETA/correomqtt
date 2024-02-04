package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginUninstallFailedEvent(String pluginId) implements Event {
}
