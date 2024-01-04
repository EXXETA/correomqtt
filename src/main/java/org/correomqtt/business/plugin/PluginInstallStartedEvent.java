package org.correomqtt.business.plugin;

import org.correomqtt.business.eventbus.Event;

public record PluginInstallStartedEvent(String pluginId, String version) implements Event {
}
