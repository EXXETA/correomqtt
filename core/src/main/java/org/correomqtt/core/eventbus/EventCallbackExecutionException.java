package org.correomqtt.core.eventbus;

import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class EventCallbackExecutionException extends RuntimeException {
    private final Exception exception;

}
