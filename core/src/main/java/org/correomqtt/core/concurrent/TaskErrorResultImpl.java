package org.correomqtt.core.concurrent;

import lombok.AllArgsConstructor;

@AllArgsConstructor
class TaskErrorResultImpl<E> {
    private E error;
    private Throwable throwable;

    boolean isExpectedImpl() {
        return error != null;
    }

    E getExpectedErrorImpl() {
        return error;
    }

    Throwable getUnexpectedErrorImpl() {
        return throwable;
    }
}
