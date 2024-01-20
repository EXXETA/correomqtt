package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginUninstallEvent(String pluginId) implements Event {
}
