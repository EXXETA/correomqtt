package org.correomqtt.business.scripting;

import org.correomqtt.business.dispatcher.ScriptResultDispatcher;

import java.io.OutputStream;

public class ScriptingLogOutputStream extends OutputStream {
    private final ExecutionDTO executionDTO;

    public ScriptingLogOutputStream(ExecutionDTO executionDTO) {
        this.executionDTO = executionDTO;
    }

    @Override
    public void write(int i) {
        executionDTO.getLog().append((char) i);
        ScriptResultDispatcher.getInstance().onScriptExecutionLogUpdate(executionDTO.getScriptExecutionDTO().getExecutionId(), (char) i);
    }

    public void append(String message) {
        for (int i = 0; i < message.length(); i++) {
            write(message.charAt(i));
        }
    }
}