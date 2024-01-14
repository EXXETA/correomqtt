package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.SimpleErrorTask;
import org.correomqtt.business.concurrent.TaskException;
import org.correomqtt.business.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MarkerFactory;

import java.io.IOException;

import static org.correomqtt.business.scripting.ScriptDeleteTask.Error.IOERROR;


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
