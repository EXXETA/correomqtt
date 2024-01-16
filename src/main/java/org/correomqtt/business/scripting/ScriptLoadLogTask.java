package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.concurrent.TaskException;
import org.correomqtt.business.fileprovider.ScriptingProvider;
import org.correomqtt.gui.views.scripting.SingleEditorViewController;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

import static org.correomqtt.business.scripting.ScriptLoadTask.Error.IOERROR;

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
