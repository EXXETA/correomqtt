package org.correomqtt.core.scripting;

import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptLoadTask.Error.IOERROR;

public class ScriptLoadLogTask extends NoProgressTask<String, ScriptLoadLogTask.Error> {


    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptLoadLogTask.class);

    private final ExecutionDTO dto;

    public enum Error {
        IOERROR
    }


    public ScriptLoadLogTask(ExecutionDTO dto) {
        this.dto = dto;
    }

    @Override
    protected String execute() {
        try {
            return ScriptingProvider.getInstance().loadLog(dto.getScriptFile().getName(), dto.getExecutionId());
        } catch (IOException e) {
            LOGGER.error("Exception loading log.", e);
            throw new TaskException(IOERROR);
        }
    }
}
