package org.correomqtt.business.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.concurrent.ConnectionTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.MessageDTO;

import java.io.File;
import java.io.IOException;

public class ImportMessageTask extends ConnectionTask<MessageDTO, Void> {

    private final File file;

    public ImportMessageTask(String connectionId, File file) {
        super(connectionId);
        this.file = file;
    }


    @Override
    protected void before() {
        EventBus.fireAsync(new ImportMessageStartedEvent(file));
    }

    @Override
    protected MessageDTO execute() throws IOException {
        return new ObjectMapper().readValue(file, MessageDTO.class);
    }

    @Override
    protected void success(MessageDTO messageDTO) {
        EventBus.fireAsync(new ImportMessageSuccessEvent(messageDTO));
    }

    @Override
    protected void error(Throwable throwable) {
        EventBus.fireAsync(new ImportMessageFailedEvent(file, throwable));
    }
}
