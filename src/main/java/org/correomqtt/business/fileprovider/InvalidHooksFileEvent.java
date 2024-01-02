package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;

import java.io.IOException;

public record InvalidHooksFileEvent(IOException ex) implements Event {
}
