package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginUninstallStartedEvent(String pluginId) implements Event {
}
