package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;

import java.io.IOException;

public record ConfigSaveFailedEvent(IOException ex) implements Event {
}
