package org.correomqtt.core.fileprovider;

import org.correomqtt.core.eventbus.Event;

public record SettingsUpdatedEvent(boolean restartRequired) implements Event {
}
