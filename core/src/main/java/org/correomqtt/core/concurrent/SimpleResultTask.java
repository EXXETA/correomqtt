package org.correomqtt.core.concurrent;

public abstract class SimpleResultTask<T> extends TaskImpl<T, Void, Void, SimpleTaskErrorResult> {

    public SimpleResultTask<T> onStarted(StartListener listener) {
        onStartedImpl(listener);
        return this;
    }

    public SimpleResultTask<T> onSuccess(SuccessListener<T> listener) {
        onSuccessImpl(listener);
        return this;
    }

    public SimpleResultTask<T> onError(SimpleTaskErrorResultListener listener) {
        onErrorImpl(errorResult -> listener.error(TaskErrorResultTransformer.implToSimple(errorResult)));
        return this;
    }

    public SimpleResultTask<T> onFinally(FinallyListener listener) {
        onFinallyImpl(listener);
        return this;
    }

    protected abstract T execute();

    @Override
    T executeImpl() throws Exception {
        return execute();
    }

    SimpleTaskErrorResult createTaskErrorResult(Void ignore, Throwable throwable) {
        return new SimpleTaskErrorResult(throwable);
    }

    protected void beforeHook() {
        // to be overridden by child on demand
    }

    protected void successHook(T result) {
        // to be overridden by child on demand
    }

    protected void errorHook(SimpleTaskErrorResult errorResult) {
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
    void errorHookImpl(SimpleTaskErrorResult errorResult) {
        errorHook(errorResult);
    }

    @Override
    void finalHookImpl() {
        finalHook();
    }

}
