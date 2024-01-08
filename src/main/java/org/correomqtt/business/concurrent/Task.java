package org.correomqtt.business.concurrent;

import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.utils.FrontendBinding;

import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;

public abstract class Task<T, P, E> {


    private final Set<SuccessListener<T>> successListener = new HashSet<>();

    private final Set<StartListener> startListener = new HashSet<>();

    private final Set<ProgressListener<P>> progressListener = new HashSet<>();


    private final Set<FinallyListener> finallyListeners = new HashSet<>();

    private final Set<ErrorListener<E>> errorListeners = new HashSet<>();
    private final Set<ErrorListenerWithException<E>> errorListenerWithExceptions = new HashSet<>();
    private final Set<ExceptionListener> exceptionListeners = new HashSet<>();
    private E expectedError;

    private void addSuccessListener(SuccessListener<T> successListener) {
        this.successListener.add(successListener);
    }

    private void addErrorListener(ErrorListener<E> errorListener) {
        this.errorListeners.add(errorListener);
    }

    private void addFinallyListener(FinallyListener finallyListener) {
        this.finallyListeners.add(finallyListener);
    }
    private void addStartListener(StartListener startListener) {
        this.startListener.add(startListener);
    }

    private void addProgressListener(ProgressListener progressListener) {
        this.progressListener.add(progressListener);
    }

    private void addErrorListener(ErrorListenerWithException<E> errorListenerWithException) {
        this.errorListenerWithExceptions.add(errorListenerWithException);
    }

    private void addErrorListener(ExceptionListener exceptionListener) {
        this.exceptionListeners.add(exceptionListener);
    }

    protected RuntimeException createExpectedException(E error) {
        this.expectedError = error;
        return new ExpectedException();
    }

    protected abstract T execute() throws Exception;

    public Task<T, P, E> onStarted(StartListener listener) {
        this.addStartListener(listener);
        return this;
    }

    public Task<T, P, E> onProgress(ProgressListener<P> listener) {
        this.addProgressListener(listener);
        return this;
    }

    public Task<T, P, E> onSuccess(SuccessListener<T> listener) {
        this.addSuccessListener(listener);
        return this;
    }

    public Task<T, P, E> onError(ErrorListener<E> listener) {
        this.addErrorListener(listener);
        return this;
    }


    public Task<T, P, E> onError(ErrorListenerWithException<E> listener) {
        this.addErrorListener(listener);
        return this;
    }


    public Task<T, P, E> onError(ExceptionListener listener) {
        this.addErrorListener(listener);
        return this;
    }

    public Task<T,P,E> onFinally(FinallyListener listener){
        this.addFinallyListener(listener);
        return this;
    }

    protected void before() {

    }

    protected void success(T result) {

    }

    protected void reportProgress(P progress) {
        progressListener.forEach(l -> FrontendBinding.pushToFrontend(() -> l.progress(progress)));
    }

    protected void error(E error, Throwable ex) {

    }

    protected void error(E error) {

    }

    protected void error(Throwable ex) {

    }

    public CompletableFuture<Void> run() {

        startListener.forEach(l -> FrontendBinding.pushToFrontend(l::start));

        before();
        return this.getFuture()
                .handleAsync((result, ex) -> {
                    if (ex != null) {
                        error(expectedError);
                        error(ex);
                        error(expectedError, ex);
                        exceptionListeners.forEach(l -> FrontendBinding.pushToFrontend(() -> l.error(ex)));
                        errorListenerWithExceptions.forEach(l -> FrontendBinding.pushToFrontend(() -> l.error(expectedError, ex)));
                        errorListeners.forEach(l -> FrontendBinding.pushToFrontend(() -> l.error(expectedError)));
                        if ((
                                !(ex instanceof ExpectedException) &&
                                        exceptionListeners.isEmpty() &&
                                        errorListenerWithExceptions.isEmpty()
                        ) || (
                                ex instanceof ExpectedException &&
                                        errorListeners.isEmpty() &&
                                        errorListenerWithExceptions.isEmpty())) {
                            EventBus.fireAsync(new UnhandledTaskExceptionEvent<E>(expectedError, ex));
                        }
                    } else {
                        success(result);
                        successListener.forEach(l -> FrontendBinding.pushToFrontend(() -> l.success(result)));
                    }
                    finallyListeners.forEach(l -> FrontendBinding.pushToFrontend(l::run));
                    return null;
                });
    }

    private CompletableFuture<T> getFuture() {

        return CompletableFuture.supplyAsync(() -> {
            try {
                return execute();
            } catch (Exception e) {
                throw new CompletionException(e);
            }
        });
    }
}
