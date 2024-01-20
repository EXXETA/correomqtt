package org.correomqtt.core.scripting;

import org.correomqtt.core.concurrent.SimpleErrorTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptSaveTask.Error.IOERROR;

public class ScriptSaveTask extends SimpleErrorTask<ScriptSaveTask.Error> {

    public enum Error {
        IOERROR
    }

    private final ScriptFileDTO scriptFileDTO;
    private final String content;

    public ScriptSaveTask(ScriptFileDTO scriptFileDTO, String content) {
        this.scriptFileDTO = scriptFileDTO;
        this.content = content;
    }

    @Override
    protected void execute() {
        try {
            ScriptingProvider.getInstance().saveScript(scriptFileDTO, content);
        } catch (IOException e) {
            throw new TaskException(IOERROR);
        }
    }
}
