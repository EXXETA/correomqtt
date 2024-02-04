package org.correomqtt.core.scripting;

public class ScriptExecutionFailedEvent extends BaseExecutionEvent {

    public ScriptExecutionFailedEvent(ExecutionDTO executionDTO) {
        super(executionDTO);
    }
}
