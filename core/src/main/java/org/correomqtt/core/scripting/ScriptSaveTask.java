package org.correomqtt.core.scripting;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleErrorTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptSaveTask.Error.IOERROR;

@DefaultBean
public class ScriptSaveTask extends SimpleErrorTask<ScriptSaveTask.Error> {

    public enum Error {
        IOERROR
    }

    private final ScriptingProvider scriptingProvider;
    private final ScriptFileDTO scriptFileDTO;
    private final String content;


    @Inject
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
