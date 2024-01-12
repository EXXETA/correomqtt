package org.correomqtt.business.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.concurrent.OnlyResultTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.MessageDTO;

import java.io.File;
import java.io.IOException;

public class ImportMessageTask extends OnlyResultTask<MessageDTO> {

    private final File file;

    public ImportMessageTask( File file) {
        this.file = file;
    }


    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new ImportMessageStartedEvent(file));
    }

    @Override
    protected MessageDTO execute() throws IOException {
        return new ObjectMapper().readValue(file, MessageDTO.class);
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
