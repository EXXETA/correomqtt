package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginInstallEvent(String pluginId, String version) implements Event {
}
