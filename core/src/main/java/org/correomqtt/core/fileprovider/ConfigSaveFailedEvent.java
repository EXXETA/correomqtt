package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Event;

import java.io.IOException;

public record ConfigSaveFailedEvent(IOException ex) implements Event {
}
