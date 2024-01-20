package org.correomqtt.core.scripting;

import org.correomqtt.core.concurrent.NoProgressTask;
import org.correomqtt.core.concurrent.TaskException;
import org.correomqtt.core.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.core.scripting.ScriptLoadTask.Error.IOERROR;

public class ScriptLoadTask extends NoProgressTask<String, ScriptLoadTask.Error> {

    public enum Error {
        IOERROR
    }

    private final ScriptFileDTO scriptFileDTO;

    public ScriptLoadTask(ScriptFileDTO scriptFileDTO) {
        this.scriptFileDTO = scriptFileDTO;
    }

    @Override
    protected String execute() {
        try {
            return ScriptingProvider.getInstance().loadScript(scriptFileDTO);
        } catch (IOException e) {
            throw new TaskException(IOERROR);
        }
    }
}
