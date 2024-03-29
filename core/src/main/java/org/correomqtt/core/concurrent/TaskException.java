package org.correomqtt.core.concurrent;

import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class TaskException extends RuntimeException {
    private final transient Object error;
}
