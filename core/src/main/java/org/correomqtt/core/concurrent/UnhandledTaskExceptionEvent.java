package org.correomqtt.core.concurrent;

import org.correomqtt.core.eventbus.Event;

public record UnhandledTaskExceptionEvent<E>(E error, Throwable ex) implements Event {
}
