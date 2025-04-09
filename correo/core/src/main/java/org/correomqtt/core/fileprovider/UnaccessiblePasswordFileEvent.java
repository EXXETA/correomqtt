package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Event;

public record UnaccessiblePasswordFileEvent(Exception ex) implements Event {
}
