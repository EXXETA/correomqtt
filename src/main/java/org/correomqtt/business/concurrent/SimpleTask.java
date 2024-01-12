package org.correomqtt.business.concurrent;

public abstract class SimpleTask extends TaskImpl<Void, Void, Void, SimpleTaskErrorResult> {

    public SimpleTask onStarted(StartListener listener) {
        onStartedImpl(listener);
        return this;
    }

    public SimpleTask onSuccess(SimpleSuccessListener listener) {
        onSuccessImpl(ignore -> listener.success());
        return this;
    }

    public SimpleTask onError(SimpleTaskErrorResultListener listener) {
        onErrorImpl(errorResult -> listener.error(TaskErrorResultTransformer.implToSimple(errorResult)));
        return this;
    }

    public SimpleTask onFinally(FinallyListener listener) {
        onFinallyImpl(listener);
        return this;
    }

    protected abstract void execute() throws Exception;

    @Override
    Void executeImpl() throws Exception {
        execute();
        return null;
    }

    SimpleTaskErrorResult createTaskErrorResult(Void ignore, Throwable throwable) {
        return new SimpleTaskErrorResult(throwable);
    }

    protected void beforeHook() {
        // to be overridden by child on demand
    }

    protected void successHook() {
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
    void successHookImpl(Void ignore) {
        successHook();
    }

    @Override
    void errorHookImpl(SimpleTaskErrorResult errorResult) {
        errorHook(TaskErrorResultTransformer.implToSimple(errorResult));
    }
}
