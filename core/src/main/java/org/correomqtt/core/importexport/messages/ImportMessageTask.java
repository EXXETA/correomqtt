package org.correomqtt.core.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleResultTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.MessageDTO;

import java.io.File;
import java.io.IOException;

public class ImportMessageTask extends SimpleResultTask<MessageDTO> {

    private final File file;


    @AssistedFactory
    public interface Factory {
        ImportMessageTask create(File file);
    }

    @AssistedInject
    public ImportMessageTask(EventBus eventBus,
                             @Assisted File file) {
        super(eventBus);
        this.file = file;
    }

    @Override
    protected MessageDTO execute() {
        try {
            return new ObjectMapper().readValue(file, MessageDTO.class);
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
    }

    @Override
    protected void beforeHook() {
        eventBus.fireAsync(new ImportMessageStartedEvent(file));
    }

    @Override
    protected void successHook(MessageDTO messageDTO) {
        eventBus.fireAsync(new ImportMessageSuccessEvent(messageDTO));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult errorResult) {
        eventBus.fireAsync(new ImportMessageFailedEvent(file, errorResult.getUnexpectedError()));
    }
}
