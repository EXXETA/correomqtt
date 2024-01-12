package org.correomqtt.business.concurrent;

public abstract class OnlyResultTask<T> extends TaskImpl<T, Void, Void, SimpleTaskErrorResult> {

    public OnlyResultTask<T> onStarted(StartListener listener) {
        onStartedImpl(listener);
        return this;
    }

    public OnlyResultTask<T> onSuccess(SuccessListener<T> listener) {
        onSuccessImpl(listener);
        return this;
    }

    public OnlyResultTask<T> onError(SimpleTaskErrorResultListener listener) {
        onErrorImpl(errorResult -> listener.error(TaskErrorResultTransformer.implToSimple(errorResult)));
        return this;
    }

    public OnlyResultTask<T> onFinally(FinallyListener listener) {
        onFinallyImpl(listener);
        return this;
    }

    protected abstract T execute() throws Exception;

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


}
