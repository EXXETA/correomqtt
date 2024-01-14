package org.correomqtt.business.concurrent;

public abstract class NoProgressTask<T, E> extends TaskImpl<T, Void, E, TaskErrorResult<E>> {

    public NoProgressTask<T, E> onStarted(StartListener listener) {
        onStartedImpl(listener);
        return this;
    }

    public NoProgressTask<T, E> onSuccess(SuccessListener<T> listener) {
        onSuccessImpl(listener);
        return this;
    }

    public NoProgressTask<T, E> onError(TaskErrorResultListener<TaskErrorResult<E>> listener) {
        onErrorImpl(listener);
        return this;
    }

    public NoProgressTask<T, E> onFinally(FinallyListener listener) {
        onFinallyImpl(listener);
        return this;
    }

    protected abstract T execute() throws Exception;

    @Override
    T executeImpl() throws Exception {
        return execute();
    }

    TaskErrorResult<E> createTaskErrorResult(E expectedError, Throwable throwable) {
        return new TaskErrorResult<>(expectedError, throwable);
    }

    protected void beforeHook() {
        // to be overridden by child on demand
    }

    protected void successHook(T result) {
        // to be overridden by child on demand
    }

    protected void errorHook(TaskErrorResult<E> errorResult) {
        // to be overridden by child on demand
    }

    protected void finalHook() {
        // to be overridden by child on demand
    }

    @Override
    void beforeHookImpl() {
        beforeHook();
    }

    @Override
    void successHookImpl(T result) {
        successHook(result);
    }

    @Override
    void errorHookImpl(TaskErrorResult<E> errorResult) {
        errorHook(errorResult);
    }

    @Override
    void finalHookImpl() {
        finalHook();
    }

}
