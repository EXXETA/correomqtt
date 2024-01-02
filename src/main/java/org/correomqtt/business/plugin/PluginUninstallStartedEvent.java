package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginUninstallStartedEvent(String pluginId) implements Event {
}
