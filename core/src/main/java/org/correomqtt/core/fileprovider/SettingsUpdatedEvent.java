package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Event;

public record SettingsUpdatedEvent(boolean restartRequired) implements Event {
}
