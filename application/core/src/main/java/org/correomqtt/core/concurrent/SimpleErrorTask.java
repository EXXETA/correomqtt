package org.correomqtt.core.concurrent;

import org.correomqtt.di.SoyEvents;

public abstract class SimpleErrorTask<E> extends TaskImpl<Void, Void, E, TaskErrorResult<E>> {

    protected SimpleErrorTask(SoyEvents soyEvents){
        super(soyEvents);
    }

    @SuppressWarnings("unused")
    public SimpleErrorTask<E> onStarted(StartListener listener) {
        onStartedImpl(listener);
        return this;
    }

    public SimpleErrorTask<E> onSuccess(SimpleSuccessListener listener) {
        onSuccessImpl(ignore -> listener.success());
        return this;
    }

    public SimpleErrorTask<E> onError(TaskErrorResultListener<TaskErrorResult<E>> listener) {
        onErrorImpl(listener);
        return this;
    }

    @SuppressWarnings("unused")
    public SimpleErrorTask<E> onFinally(FinallyListener listener) {
        onFinallyImpl(listener);
        return this;
    }

    protected abstract void execute();

    @Override
    Void executeImpl() {
        execute();
        return null;
    }

    TaskErrorResult<E> createTaskErrorResult(E expectedError, Throwable throwable) {
        return new TaskErrorResult<>(expectedError, throwable);
    }

    protected void beforeHook() {
        // to be overridden by child on demand
    }

    protected void successHook() {
        // to be overridden by child on demand
    }

    @SuppressWarnings("unused")
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
    void successHookImpl(Void ignore) {
        successHook();
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
