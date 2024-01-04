package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginUninstallEvent(String pluginId) implements Event {
}
