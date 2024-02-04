package org.correomqtt.core.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleResultTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.model.MessageDTO;

import java.io.File;
import java.io.IOException;

@DefaultBean
public class ImportMessageTask extends SimpleResultTask<MessageDTO> {

    private final File file;




    @Inject
    public ImportMessageTask(SoyEvents soyEvents,
                             @Assisted File file) {
        super(soyEvents);
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
        soyEvents.fireAsync(new ImportMessageStartedEvent(file));
    }

    @Override
    protected void successHook(MessageDTO messageDTO) {
        soyEvents.fireAsync(new ImportMessageSuccessEvent(messageDTO));
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult errorResult) {
        soyEvents.fireAsync(new ImportMessageFailedEvent(file, errorResult.getUnexpectedError()));
    }
}
