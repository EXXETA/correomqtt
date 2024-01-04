package org.correomqtt.business.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.concurrent.ConnectionTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.MessageDTO;

import java.io.File;
import java.io.IOException;

public class ExportMessageTask extends ConnectionTask<Void, Void> {

    private final File file;
    private final MessageDTO messageDTO;

    public ExportMessageTask(String connectionId, File file, MessageDTO messageDTO) {
        super(connectionId);
        this.file = file;
        this.messageDTO = messageDTO;
    }

    @Override
    protected void before() {
        EventBus.fireAsync(new ExportMessageStartedEvent(file, messageDTO));
    }

    @Override
    protected Void execute() throws IOException {
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(file, messageDTO);
        return null;
    }

    @Override
    protected void success(Void result) {
        EventBus.fireAsync(new ExportMessageSuccessEvent());
    }

    @Override
    protected void error(Throwable throwable) {
        EventBus.fireAsync(new ExportMessageFailedEvent(file, messageDTO, throwable));
    }
}
