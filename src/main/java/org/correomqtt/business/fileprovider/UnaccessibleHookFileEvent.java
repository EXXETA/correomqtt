package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;

public record UnaccessibleHookFileEvent(Exception ex) implements Event {
}
