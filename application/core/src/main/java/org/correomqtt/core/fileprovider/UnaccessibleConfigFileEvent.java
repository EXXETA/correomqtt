package org.correomqtt.core.fileprovider;

import org.correomqtt.di.Event;

public record UnaccessibleConfigFileEvent(Exception ex) implements Event {
}
