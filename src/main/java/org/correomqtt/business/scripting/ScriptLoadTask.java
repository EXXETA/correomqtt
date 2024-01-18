package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.NoProgressTask;
import org.correomqtt.business.concurrent.TaskException;
import org.correomqtt.business.fileprovider.ScriptingProvider;

import java.io.IOException;

import static org.correomqtt.business.scripting.ScriptLoadTask.Error.IOERROR;

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
