package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;

public record DirectoryCanNotBeCreatedEvent(String path) implements Event {
}
