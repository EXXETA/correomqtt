package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptExecutionDTO;

public interface ExecuteScriptObserver extends BaseObserver {
    void onExecuteScriptSucceeded(ScriptExecutionDTO scriptExecutionDTO, long executionTimeInMilliseconds);

    void onExecuteScriptCancelled(ScriptExecutionDTO scriptExecutionDTO);

    void onExecuteScriptFailed(ScriptExecutionDTO scriptExecutionDTO, Throwable exception);

    default void onExecuteScriptRunning(ScriptExecutionDTO scriptExecutionDTO) {
        // nothing to do
    }

    default void onExecuteScriptScheduled(ScriptExecutionDTO scriptExecutionDTOt) {
        // nothing to do
    }

    default void onExecuteScriptStarted(ScriptExecutionDTO scriptExecutionDTO) {
        // nothing to do
    }
}
