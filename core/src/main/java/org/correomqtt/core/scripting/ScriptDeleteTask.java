package org.correomqtt.core.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleErrorTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MarkerFactory;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptDeleteTask.Error.IOERROR;


public class ScriptDeleteTask extends SimpleErrorTask<ScriptDeleteTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptDeleteTask.class);

    public enum Error {
        IOERROR
    }

    private final ScriptingProvider scriptingProvider;
    private final ScriptFileDTO dto;

    @AssistedFactory
    public interface Factory {
        ScriptDeleteTask create(ScriptFileDTO dto);
    }

    @AssistedInject
    public ScriptDeleteTask(ScriptingProvider scriptingProvider,
                            @Assisted ScriptFileDTO dto) {
        this.scriptingProvider = scriptingProvider;
        this.dto = dto;
    }

    @Override
    protected void execute() throws TaskException {

        String ioErrorMsg = "Error delete script. ";

        try {
            ScriptingBackend.removeExecutionsForScript(dto.getName());
            scriptingProvider.deleteScript(dto.getName());
        } catch (IOException e) {
            LOGGER.error(MarkerFactory.getMarker(dto.getName()), ioErrorMsg, e);
            throw new TaskException(IOERROR);
        }
    }
}
