package org.correomqtt.core.concurrent;

import org.correomqtt.di.SoyEvents;

public abstract class SimpleProgressTask<P> extends TaskImpl<Void, P, Void, SimpleTaskErrorResult> {

    protected SimpleProgressTask(SoyEvents soyEvents) {
        super(soyEvents);
    }

    public SimpleProgressTask<P> onStarted(StartListener listener) {
        onStartedImpl(listener);
        return this;
    }

    public SimpleProgressTask<P> onProgress(ProgressListener<P> listener) {
        onProgressImpl(listener);
        return this;
    }

    public SimpleProgressTask<P> onSuccess(SimpleSuccessListener listener) {
        onSuccessImpl(ignore -> listener.success());

        return this;
    }

    public SimpleProgressTask<P> onError(TaskErrorResultListener<SimpleTaskErrorResult> listener) {
        onErrorImpl(listener);
        return this;
    }

    public SimpleProgressTask<P> onFinally(FinallyListener listener) {
        onFinallyImpl(listener);
        return this;
    }

    protected void reportProgress(P progress) {
        reportProgressImpl(progress);
    }

    protected abstract void execute();

    @Override
    Void executeImpl() {
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

    protected void finalHook() {
        // to be overridden by child on demand
    }

    @Override
    void beforeHookImpl() {
        beforeHook();
    }

    @Override
    void successHookImpl(Void result) {
        successHook();
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
