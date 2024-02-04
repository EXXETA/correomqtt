package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Event;

public record UnaccessibleHookFileEvent(Exception ex) implements Event {
}
