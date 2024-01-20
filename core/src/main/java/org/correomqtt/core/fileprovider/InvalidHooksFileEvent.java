package org.correomqtt.core.fileprovider;

import org.correomqtt.core.eventbus.Event;

import java.io.IOException;

public record InvalidHooksFileEvent(IOException ex) implements Event {
}
