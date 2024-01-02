package org.correomqtt.business.fileprovider;

import org.correomqtt.business.eventbus.Event;

public record UnaccessibleConfigFileEvent(Exception ex) implements Event {
}
