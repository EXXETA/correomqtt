package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginUninstallFailedEvent(String pluginId) implements Event {
}
