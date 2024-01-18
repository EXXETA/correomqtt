package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.SimpleErrorTask;
import org.correomqtt.business.concurrent.TaskException;
import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

import static org.correomqtt.business.scripting.ScriptLoadTask.Error.IOERROR;

public class ScriptDeleteExecutionsTask extends SimpleErrorTask<ScriptDeleteExecutionsTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptDeleteExecutionsTask.class);

    private final String filename;

    public enum Error {
        IOERROR
    }


    public ScriptDeleteExecutionsTask(String filename) {
        this.filename = filename;
    }

    @Override
    protected void execute() {
        try {
            ScriptingBackend.removeExecutionsForScript(filename);
            ScriptingProvider.getInstance().deleteExecutions(filename);
            EventBus.fireAsync(new ScriptExecutionsDeletedEvent(filename));
        } catch (IOException e) {
            LOGGER.error("Exception removing executions for {}.", filename, e);
            throw new TaskException(IOERROR);
        }
    }
}
