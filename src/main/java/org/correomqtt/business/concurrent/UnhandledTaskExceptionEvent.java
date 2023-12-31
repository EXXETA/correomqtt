package org.correomqtt.business.concurrent;

import lombok.Getter;
import org.correomqtt.business.eventbus.Event;

public record UnhandledTaskExceptionEvent<E>(E error, Throwable ex) implements Event {
}
