package org.correomqtt.core.scripting;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleErrorTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptLoadTask.Error.IOERROR;

@DefaultBean
public class ScriptDeleteExecutionsTask extends SimpleErrorTask<ScriptDeleteExecutionsTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptDeleteExecutionsTask.class);

    private final ScriptingProvider scriptingProvider;
    private final SoyEvents soyEvents;
    private final String filename;

    public enum Error {
        IOERROR
    }



    @Inject
    public ScriptDeleteExecutionsTask(ScriptingProvider scriptingProvider,
                                      SoyEvents soyEvents,
                                      @Assisted String filename) {
        super(soyEvents);
        this.scriptingProvider = scriptingProvider;
        this.soyEvents = soyEvents;
        this.filename = filename;
    }

    @Override
    protected void execute() {
        try {
            ScriptingBackend.removeExecutionsForScript(filename);
            scriptingProvider.deleteExecutions(filename);
            soyEvents.fireAsync(new ScriptExecutionsDeletedEvent(filename));
        } catch (IOException e) {
            LOGGER.error("Exception removing executions for {}.", filename, e);
            throw new TaskException(IOERROR);
        }
    }
}
