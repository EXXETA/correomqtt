package org.correomqtt.business.scripting;

import java.io.OutputStream;

public class ScriptingLogOutputStream extends OutputStream {
    private final ExecutionDTO executionDTO;

    public ScriptingLogOutputStream(ExecutionDTO executionDTO) {
        this.executionDTO = executionDTO;
    }

    @Override
    public void write(int i) {
        executionDTO.getLog().append((char) i);
    }

}