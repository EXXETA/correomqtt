package org.correomqtt.core.fileprovider;

import org.correomqtt.core.eventbus.Event;

public record DirectoryCanNotBeCreatedEvent(String path) implements Event {
}
