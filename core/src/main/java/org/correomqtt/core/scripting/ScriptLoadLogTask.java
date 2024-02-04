package org.correomqtt.core.scripting;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.di.SoyEvents;
import org.correomqtt.core.fileprovider.ScriptingProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptLoadTask.Error.IOERROR;

@DefaultBean
public class ScriptLoadLogTask extends NoProgressTask<String, ScriptLoadLogTask.Error> {


    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptLoadLogTask.class);

    private final ScriptingProvider scriptingProvider;
    private final ExecutionDTO dto;

    public enum Error {
        IOERROR
    }



    @Inject
    public ScriptLoadLogTask(ScriptingProvider scriptingProvider,
                             SoyEvents soyEvents,
                             @Assisted ExecutionDTO dto) {
        super(soyEvents);
        this.scriptingProvider = scriptingProvider;
        this.dto = dto;
    }

    @Override
    protected String execute() {
        try {
            return scriptingProvider.loadLog(dto.getScriptFile().getName(), dto.getExecutionId());
        } catch (IOException e) {
            LOGGER.error("Exception loading log.", e);
            throw new TaskException(IOERROR);
        }
    }
}
