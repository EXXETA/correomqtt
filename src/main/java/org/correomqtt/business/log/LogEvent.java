package org.correomqtt.business.log;

import org.correomqtt.business.eventbus.Event;

public record LogEvent(String logMsg) implements Event {
}
