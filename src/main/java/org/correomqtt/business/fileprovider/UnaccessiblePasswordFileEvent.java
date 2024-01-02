package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;

public record UnaccessiblePasswordFileEvent(Exception ex) implements Event {
}
