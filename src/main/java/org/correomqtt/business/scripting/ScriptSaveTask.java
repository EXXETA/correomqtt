package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.OnlyErrorTask;
import org.correomqtt.business.concurrent.TaskException;
import org.correomqtt.business.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.business.scripting.ScriptSaveTask.Error.IOERROR;

public class ScriptSaveTask extends OnlyErrorTask<ScriptSaveTask.Error> {

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
