package org.correomqtt.core.fileprovider;

import org.correomqtt.core.eventbus.Event;

public record UnaccessibleConfigFileEvent(Exception ex) implements Event {
}
