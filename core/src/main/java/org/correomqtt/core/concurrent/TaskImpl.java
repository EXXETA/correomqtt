package org.correomqtt.core.concurrent;

import org.correomqtt.di.SoyEvents;
import org.correomqtt.di.FrontendBinding;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;

abstract class TaskImpl<T, P, E, R> {

    protected final SoyEvents soyEvents;

    protected TaskImpl(SoyEvents soyEvents){
        this.soyEvents = soyEvents;
    }

    abstract T executeImpl() throws Exception;

    abstract void beforeHookImpl();

    abstract void successHookImpl(T result);

    abstract void errorHookImpl(R errorResult);

    abstract void finalHookImpl();

    private static final Logger LOGGER = LoggerFactory.getLogger(TaskImpl.class);
    private final Set<SuccessListener<T>> successListener = new HashSet<>();
    private final Set<StartListener> startListener = new HashSet<>();
    private final Set<ProgressListener<P>> progressListener = new HashSet<>();
    private final Set<FinallyListener> finallyListener = new HashSet<>();
    private final Set<TaskErrorResultListener<R>> errorListener = new HashSet<>();

    void onStartedImpl(StartListener listener) {
        this.startListener.add(listener);
    }

    void onProgressImpl(ProgressListener<P> listener) {
        this.progressListener.add(listener);
    }

    void onSuccessImpl(SuccessListener<T> listener) {
        this.successListener.add(listener);
    }

    void onErrorImpl(TaskErrorResultListener<R> listener) {
        this.errorListener.add(listener);
    }

    void onFinallyImpl(FinallyListener listener) {
        this.finallyListener.add(listener);
    }

    void reportProgressImpl(P progress) {
        progressListener.forEach(l -> FrontendBinding.pushToFrontend(() -> l.progress(progress)));
    }

    @SuppressWarnings("unchecked")
    public CompletableFuture<Void> run() {

        startListener.forEach(l -> FrontendBinding.pushToFrontend(l::start));

        if (errorListener.isEmpty()) {
            LOGGER.warn("You executed {} without providing ExceptionListener. While unexpected Exceptions will be logged please consider adding custom error handling.", this.getClass());
        }

        beforeHookImpl();
        return this.getFuture()
                .handleAsync((result, t) -> {
                    if (t != null) {
                        E expectedError = null;
                        if (t instanceof CompletionException ce && ce.getCause() instanceof TaskException taskException) {
                            expectedError = (E) taskException.getError();
                        }
                        if (errorListener.isEmpty()) {
                            LOGGER.error("Unhandled exeception executing Task. Please consider using an ExceptionListener. ", t);
                            soyEvents.fireAsync(new UnhandledTaskExceptionEvent<E>(expectedError, t));
                        }
                        R errorResult = createTaskErrorResult(expectedError, t);
                        errorHookImpl(errorResult);
                        errorListener.forEach(l -> FrontendBinding.pushToFrontend(() -> l.error(errorResult)));
                    } else {
                        successHookImpl(result);
                        successListener.forEach(l -> FrontendBinding.pushToFrontend(() -> l.success(result)));
                    }
                    finalHookImpl();
                    finallyListener.forEach(l -> FrontendBinding.pushToFrontend(l::run));
                    return null;
                });
    }


    abstract R createTaskErrorResult(E expectedError, Throwable throwable);

    private CompletableFuture<T> getFuture() {

        return CompletableFuture.supplyAsync(() -> {
            try {
                return executeImpl();
            } catch (Exception e) {
                throw new CompletionException(e);
            }
        });
    }
}
