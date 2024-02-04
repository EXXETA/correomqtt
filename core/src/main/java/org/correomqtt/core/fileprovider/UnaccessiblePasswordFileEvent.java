package org.correomqtt.core.fileprovider;

import org.correomqtt.core.eventbus.Event;

public record UnaccessiblePasswordFileEvent(Exception ex) implements Event {
}
