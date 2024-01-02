package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginInstallFailedEvent(String pluginId, String version) implements Event {
}
