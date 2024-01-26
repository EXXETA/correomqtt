package org.correomqtt.core.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleErrorTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptSaveTask.Error.IOERROR;

public class ScriptSaveTask extends SimpleErrorTask<ScriptSaveTask.Error> {

    public enum Error {
        IOERROR
    }

    private final ScriptingProvider scriptingProvider;
    private final ScriptFileDTO scriptFileDTO;
    private final String content;

    @AssistedFactory
    public interface Factory {
        ScriptSaveTask create(ScriptFileDTO scriptFileDTO, String contente);
    }

    @AssistedInject
    public ScriptSaveTask(ScriptingProvider scriptingProvider,
                          EventBus eventBus,
                          @Assisted ScriptFileDTO scriptFileDTO,
                          @Assisted String content) {
        super(eventBus);
        this.scriptingProvider = scriptingProvider;
        this.scriptFileDTO = scriptFileDTO;
        this.content = content;
    }

    @Override
    protected void execute() {
        try {
            scriptingProvider.saveScript(scriptFileDTO, content);
        } catch (IOException e) {
            throw new TaskException(IOERROR);
        }
    }
}
