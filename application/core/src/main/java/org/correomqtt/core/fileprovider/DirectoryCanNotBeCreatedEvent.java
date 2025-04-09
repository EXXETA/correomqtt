package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Event;

public record DirectoryCanNotBeCreatedEvent(String path) implements Event {
}
