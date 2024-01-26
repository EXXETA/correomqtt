package org.correomqtt.core.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.MessageDTO;

import java.io.File;
import java.io.IOException;

public class ExportMessageTask extends SimpleTask {

    private final EventBus eventBus;
    private final File file;
    private final MessageDTO messageDTO;

    @AssistedFactory
    public interface Factory {
        ExportMessageTask create(File file, MessageDTO messageDTO);
    }

    @AssistedInject
    public ExportMessageTask(EventBus eventBus,
                             @Assisted File file,
                             @Assisted MessageDTO messageDTO) {
        super(eventBus);
        this.eventBus = eventBus;
        this.file = file;
        this.messageDTO = messageDTO;
    }

    @Override
    protected void beforeHook() {
        eventBus.fireAsync(new ExportMessageStartedEvent(file, messageDTO));
    }

    @Override
    protected void execute() {
        try {
            new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(file, messageDTO);
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
    }

    @Override
    protected void successHook() {
        eventBus.fireAsync(new ExportMessageSuccessEvent());
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult errorResult) {
        eventBus.fireAsync(new ExportMessageFailedEvent(file, messageDTO, errorResult.getUnexpectedError()));
    }
}
