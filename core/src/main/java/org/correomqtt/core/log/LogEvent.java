package org.correomqtt.core.log;

import org.correomqtt.di.Event;

public record LogEvent(String logMsg) implements Event {
    @Override
    public boolean isLogable() {
        return false;
    }
}
