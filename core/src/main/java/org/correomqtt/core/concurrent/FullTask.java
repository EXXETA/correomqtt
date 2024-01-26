package org.correomqtt.core.concurrent;

import org.correomqtt.core.eventbus.EventBus;

public abstract class FullTask<T, P, E> extends TaskImpl<T, P, E, TaskErrorResult<E>> {

    protected FullTask(EventBus eventBus) {
        super(eventBus);
    }

    public FullTask<T, P, E> onStarted(StartListener listener) {
        onStartedImpl(listener);
        return this;
    }

    public FullTask<T, P, E> onProgress(ProgressListener<P> listener) {
        onProgressImpl(listener);
        return this;
    }

    public FullTask<T, P, E> onSuccess(SuccessListener<T> listener) {
        onSuccessImpl(listener);
        return this;
    }

    public FullTask<T, P, E> onError(TaskErrorResultListener<TaskErrorResult<E>> listener) {
        onErrorImpl(listener);
        return this;
    }

    public FullTask<T, P, E> onFinally(FinallyListener listener) {
        onFinallyImpl(listener);
        return this;
    }

    protected void reportProgress(P progress) {
        reportProgressImpl(progress);
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
