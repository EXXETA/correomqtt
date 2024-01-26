package org.correomqtt.core.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleErrorTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptLoadTask.Error.IOERROR;

public class ScriptDeleteExecutionsTask extends SimpleErrorTask<ScriptDeleteExecutionsTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptDeleteExecutionsTask.class);

    private final ScriptingProvider scriptingProvider;
    private final EventBus eventBus;
    private final String filename;

    public enum Error {
        IOERROR
    }

    @AssistedFactory
    public interface Factory {
        ScriptDeleteExecutionsTask create(String filename);
    }

    @AssistedInject
    public ScriptDeleteExecutionsTask(ScriptingProvider scriptingProvider,
                                      EventBus eventBus,
                                      @Assisted String filename) {
        super(eventBus);
        this.scriptingProvider = scriptingProvider;
        this.eventBus = eventBus;
        this.filename = filename;
    }

    @Override
    protected void execute() {
        try {
            ScriptingBackend.removeExecutionsForScript(filename);
            scriptingProvider.deleteExecutions(filename);
            eventBus.fireAsync(new ScriptExecutionsDeletedEvent(filename));
        } catch (IOException e) {
            LOGGER.error("Exception removing executions for {}.", filename, e);
            throw new TaskException(IOERROR);
        }
    }
}
