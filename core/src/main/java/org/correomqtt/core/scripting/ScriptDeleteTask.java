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
import org.slf4j.MarkerFactory;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptDeleteTask.Error.IOERROR;


@DefaultBean
public class ScriptDeleteTask extends SimpleErrorTask<ScriptDeleteTask.Error> {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptDeleteTask.class);

    public enum Error {
        IOERROR
    }

    private final ScriptingProvider scriptingProvider;
    private final ScriptFileDTO dto;



    @Inject
    public ScriptDeleteTask(ScriptingProvider scriptingProvider,
                            SoyEvents soyEvents,
                            @Assisted ScriptFileDTO dto) {
        super(soyEvents);
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
