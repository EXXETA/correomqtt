package org.correomqtt.core.scripting;

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

    private final ScriptFileDTO dto;

    public ScriptDeleteTask(ScriptFileDTO dto) {
        this.dto = dto;
    }

    @Override
    protected void execute() throws TaskException {

        String ioErrorMsg = "Error delete script. ";

        try {
            ScriptingBackend.removeExecutionsForScript(dto.getName());
            ScriptingProvider.getInstance().deleteScript(dto.getName());
        } catch (IOException e) {
            LOGGER.error(MarkerFactory.getMarker(dto.getName()), ioErrorMsg, e);
            throw new TaskException(IOERROR);
        }
    }
}
