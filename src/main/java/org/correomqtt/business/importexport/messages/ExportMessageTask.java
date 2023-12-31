package org.correomqtt.business.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.business.concurrent.ConnectionTask;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.correomqtt.business.model.MessageDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;

public class ExportMessageTask extends ConnectionTask<Void, Void> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ExportMessageTask.class);

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
