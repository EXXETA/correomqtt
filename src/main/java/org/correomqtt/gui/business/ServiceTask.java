package org.correomqtt.gui.business;

import org.correomqtt.business.services.BusinessService;
import javafx.concurrent.Task;

import java.util.function.Consumer;

class ServiceTask<S extends BusinessService> extends Task<Void> {

    private final S backendService;
    private final Consumer<S> consumer;

    ServiceTask(S backendService, Consumer<S> consumer) {
        this.backendService = backendService;
        this.consumer = consumer;
    }

    @Override
    protected Void call() {
        consumer.accept(backendService);
        return null;
    }

    @Override
    protected void succeeded() {
        backendService.onSucceeded();
    }

    @Override
    protected void cancelled() {
        backendService.onCancelled();
    }

    @Override
    protected void failed() {
        backendService.onFailed(super.getException());
    }

    @Override
    protected void running() {
        backendService.onRunning();
    }

    @Override
    protected void scheduled() {
        backendService.onScheduled();
    }
}
