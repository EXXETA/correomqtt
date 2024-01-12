package org.correomqtt.business.concurrent;

public abstract class OnlyErrorTask<E> extends TaskImpl<Void, Void, E, TaskErrorResult<E>> {

    public OnlyErrorTask<E> onStarted(StartListener listener) {
        onStartedImpl(listener);
        return this;
    }

    public OnlyErrorTask<E> onSuccess(SimpleSuccessListener listener) {
        onSuccessImpl(ignore -> listener.success());
        return this;
    }

    public OnlyErrorTask<E> onError(TaskErrorResultListener<TaskErrorResult<E>> listener) {
        onErrorImpl(listener);
        return this;
    }

    public OnlyErrorTask<E> onFinally(FinallyListener listener) {
        onFinallyImpl(listener);
        return this;
    }

    protected abstract void execute() throws Exception;

    @Override
    Void executeImpl() throws Exception {
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

    protected void errorHook(TaskErrorResult<E> errorResult) {
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

}
