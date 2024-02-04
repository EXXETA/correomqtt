package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.core.eventbus.Event;

public record PluginInstallEvent(String pluginId, String version) implements Event {
}
