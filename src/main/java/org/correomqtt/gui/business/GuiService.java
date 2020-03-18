package org.correomqtt.gui.business;

import org.correomqtt.business.services.BusinessService;
import javafx.concurrent.Service;
import javafx.concurrent.Task;

import java.util.function.Consumer;

public class GuiService<S extends BusinessService>  extends Service<Void> {

    private final S backendService;
    private final Consumer<S> consumer;

    GuiService(S backendService, Consumer<S> consumer) {
        this.backendService = backendService;
        this.consumer = consumer;
    }

    @Override
    protected Task<Void> createTask() {
        return new ServiceTask<>(backendService, consumer);
    }
}
