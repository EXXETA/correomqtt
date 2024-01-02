package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;

public record SettingsUpdatedEvent(boolean restartRequired) implements Event {
}
