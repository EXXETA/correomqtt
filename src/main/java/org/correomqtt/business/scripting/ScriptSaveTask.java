package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.business.scripting.ScriptSaveTask.Error.IOERROR;

public class ScriptSaveTask extends NoProgressTask<Void, ScriptSaveTask.Error> {

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
    protected Void execute() {
        try {
            ScriptingProvider.getInstance().saveScript(scriptFileDTO, content);
            return null;
        } catch (IOException e) {
            throw createExpectedException(IOERROR);
        }
    }
}
