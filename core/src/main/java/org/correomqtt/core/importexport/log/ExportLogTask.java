package org.correomqtt.core.importexport.log;

import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.concurrent.SimpleTaskErrorResult;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.di.SoyEvents;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;

@DefaultBean
public class ExportLogTask extends SimpleTask {

    private final SoyEvents soyEvents;

    private final File file;

    private final String logs;

    @Inject
    public ExportLogTask(SoyEvents soyEvents,
                         @Assisted File file,
                         @Assisted String logs) {
        super(soyEvents);
        this.soyEvents = soyEvents;
        this.file = file;
        this.logs = logs;
    }

    @Override
    protected void beforeHook() {
        soyEvents.fireAsync(new ExportLogStartedEvent(file, logs));
    }

    @Override
    protected void execute() {
        try {
            Files.writeString(file.toPath(), logs);
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
    }

    @Override
    protected void successHook() {
        soyEvents.fireAsync(new ExportLogSuccessEvent());
    }

    @Override
    protected void errorHook(SimpleTaskErrorResult errorResult) {
        soyEvents.fireAsync(new ExportLogFailedEvent(file, logs, errorResult.getUnexpectedError()));
    }
}
