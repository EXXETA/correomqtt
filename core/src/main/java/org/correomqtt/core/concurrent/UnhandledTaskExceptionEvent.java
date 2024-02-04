package org.correomqtt.core.concurrent;

import org.correomqtt.di.Event;

public record UnhandledTaskExceptionEvent<E>(E error, Throwable ex) implements Event {
}
