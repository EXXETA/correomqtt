package org.correomqtt.core.plugin.marketplace;

import org.correomqtt.di.Event;

public record PluginInstallFailedEvent(String pluginId, String version) implements Event {
}
