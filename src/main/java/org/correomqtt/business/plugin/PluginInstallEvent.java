package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginInstallEvent(String pluginId, String version) implements Event {
}
