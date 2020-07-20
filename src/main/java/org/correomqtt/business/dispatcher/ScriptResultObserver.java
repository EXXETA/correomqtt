package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptExecutionDTO;

public interface ScriptResultObserver extends BaseObserver {

    void onScriptExecutionSucceeded(ScriptExecutionDTO scriptExecutionDTO, long executionTimeInMilliseconds);

    void onScriptExecutionTimeout(ScriptExecutionDTO scriptExecutionDTO);

    void onScriptExecutionCancelled(ScriptExecutionDTO scriptExecutionDTO);

    void onScriptExecutionFailed(ScriptExecutionDTO scriptExecutionDTO, long executionTimeInMilliseconds, Throwable t);
}
