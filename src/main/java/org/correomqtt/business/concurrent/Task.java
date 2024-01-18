package org.correomqtt.business.concurrent;

import java.util.concurrent.CompletableFuture;

public interface Task<T, P, E> {

    FullTask<T, P, E> onStarted(StartListener listener);

    FullTask<T, P, E> onProgress(ProgressListener<P> listener);

    FullTask<T, P, E> onSuccess(SuccessListener<T> listener);

    FullTask<T, P, E> onError(TaskErrorResultListener<TaskErrorResult<E>> listener);

    FullTask<T, P, E> onFinally(FinallyListener listener);

    CompletableFuture<Void> run();
}
