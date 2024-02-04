package org.correomqtt.core.log;

import org.correomqtt.core.eventbus.Event;

public record LogEvent(String logMsg) implements Event {
}
