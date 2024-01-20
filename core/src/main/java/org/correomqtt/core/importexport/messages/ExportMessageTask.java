package org.correomqtt.core.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.MessageDTO;

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
    protected void execute() {
        try {
            new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(file, messageDTO);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
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
