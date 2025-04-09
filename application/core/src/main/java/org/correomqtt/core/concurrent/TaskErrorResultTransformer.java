package org.correomqtt.core.concurrent;

public class TaskErrorResultTransformer {

    private TaskErrorResultTransformer() {
        // private constructor
    }

    public static <E> SimpleTaskErrorResult implToSimple(TaskErrorResultImpl<E> result) {
        return new SimpleTaskErrorResult(result.getUnexpectedErrorImpl());
    }

    public static <E> TaskErrorResult<E> implToResult(TaskErrorResultImpl<E> result) {
        return new TaskErrorResult<>(result.getExpectedErrorImpl(), result.getUnexpectedErrorImpl());
    }
}
