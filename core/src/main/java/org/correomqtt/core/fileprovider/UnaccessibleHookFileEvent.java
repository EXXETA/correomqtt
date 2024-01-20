package org.correomqtt.core.fileprovider;

import org.correomqtt.core.eventbus.Event;

public record UnaccessibleHookFileEvent(Exception ex) implements Event {
}
