package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginUninstallStartedEvent(String pluginId) implements Event {
}
