package org.correomqtt.core.concurrent;

public class TaskErrorResult<E> extends TaskErrorResultImpl<E> {
    public TaskErrorResult(E error, Throwable throwable) {
        super(error, throwable);
    }

    public boolean isExpected() {
        return isExpectedImpl();
    }

    public E getExpectedError() {
        return getExpectedErrorImpl();
    }

    public Throwable getUnexpectedError() {
        return getUnexpectedErrorImpl();
    }
}
