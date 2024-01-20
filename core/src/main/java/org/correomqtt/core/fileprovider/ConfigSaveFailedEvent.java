package org.correomqtt.core.fileprovider;

import org.correomqtt.core.eventbus.Event;

import java.io.IOException;

public record ConfigSaveFailedEvent(IOException ex) implements Event {
}
