package org.correomqtt.business.eventbus;

import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class EventCallbackExecutionException extends RuntimeException {
    private final Exception exception;

}
