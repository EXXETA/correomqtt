package org.correomqtt.core.scripting;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptLoadTask.Error.IOERROR;

@DefaultBean
public class ScriptLoadTask extends NoProgressTask<String, ScriptLoadTask.Error> {

    public enum Error {
        IOERROR
    }

    private final ScriptingProvider scriptingProvider;
    private final ScriptFileDTO scriptFileDTO;


    @Inject
    public ScriptLoadTask(ScriptingProvider scriptingProvider,
                          EventBus eventBus,
                          @Assisted ScriptFileDTO scriptFileDTO) {
        super(eventBus);
        this.scriptingProvider = scriptingProvider;
        this.scriptFileDTO = scriptFileDTO;
    }

    @Override
    protected String execute() {
        try {
            return scriptingProvider.loadScript(scriptFileDTO);
        } catch (IOException e) {
            throw new TaskException(IOERROR);
        }
    }
}
