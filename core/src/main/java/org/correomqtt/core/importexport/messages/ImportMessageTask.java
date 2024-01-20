package org.correomqtt.core.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.core.concurrent.SimpleResultTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.MessageDTO;

import java.io.File;
import java.io.IOException;

public class ImportMessageTask extends SimpleResultTask<MessageDTO> {

    private final File file;

    public ImportMessageTask( File file) {
        this.file = file;
    }


    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new ImportMessageStartedEvent(file));
    }

    @Override
    protected MessageDTO execute() {
        try {
            return new ObjectMapper().readValue(file, MessageDTO.class);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    protected void successHook(MessageDTO messageDTO) {
        EventBus.fireAsync(new ImportMessageSuccessEvent(messageDTO));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult errorResult) {
        EventBus.fireAsync(new ImportMessageFailedEvent(file, errorResult.getUnexpectedError()));
    }
}
