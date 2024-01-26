package org.correomqtt.core.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptLoadTask.Error.IOERROR;

public class ScriptLoadTask extends NoProgressTask<String, ScriptLoadTask.Error> {

    public enum Error {
        IOERROR
    }

    private final ScriptingProvider scriptingProvider;
    private final ScriptFileDTO scriptFileDTO;

    @AssistedFactory
    public interface Factory {
        ScriptLoadTask create(ScriptFileDTO scriptFileDTO);
    }

    @AssistedInject
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
