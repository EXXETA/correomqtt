package org.correomqtt.core.importexport.messages;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.model.MessageDTO;

import java.io.File;
import java.io.IOException;

@DefaultBean
public class ExportMessageTask extends SimpleTask {

    private final EventBus eventBus;
    private final File file;
    private final MessageDTO messageDTO;



    @Inject
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
