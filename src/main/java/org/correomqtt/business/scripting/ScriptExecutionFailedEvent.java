package org.correomqtt.business.scripting;

public class ScriptExecutionFailedEvent extends BaseExecutionEvent {

    public ScriptExecutionFailedEvent(ExecutionDTO executionDTO) {
        super(executionDTO);
    }
}
