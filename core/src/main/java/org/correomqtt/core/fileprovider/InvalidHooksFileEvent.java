package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Event;

import java.io.IOException;

public record InvalidHooksFileEvent(IOException ex) implements Event {
}
