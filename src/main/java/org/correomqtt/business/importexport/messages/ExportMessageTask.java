package org.correomqtt.business.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.concurrent.SimpleTask;
import org.correomqtt.business.concurrent.SimpleTaskErrorResult;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.model.MessageDTO;

import java.io.File;
import java.io.IOException;

public class ExportMessageTask extends SimpleTask {

    private final File file;
    private final MessageDTO messageDTO;

    public ExportMessageTask(File file, MessageDTO messageDTO) {
        this.file = file;
        this.messageDTO = messageDTO;
    }

    @Override
    protected void beforeHook() {
        EventBus.fireAsync(new ExportMessageStartedEvent(file, messageDTO));
    }

    @Override
    protected void execute() throws IOException {
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(file, messageDTO);
    }

    @Override
    protected void successHook() {
        EventBus.fireAsync(new ExportMessageSuccessEvent());
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult errorResult) {
        EventBus.fireAsync(new ExportMessageFailedEvent(file, messageDTO, errorResult.getUnexpectedError()));
    }
}
