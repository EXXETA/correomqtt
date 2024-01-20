package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginInstallStartedEvent(String pluginId, String version) implements Event {
}
