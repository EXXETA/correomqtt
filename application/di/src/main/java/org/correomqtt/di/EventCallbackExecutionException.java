package org.correomqtt.di;

import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class EventCallbackExecutionException extends RuntimeException {
    private final Exception exception;

}
