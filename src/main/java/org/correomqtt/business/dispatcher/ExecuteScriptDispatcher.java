package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptExecutionDTO;

public class ExecuteScriptDispatcher extends BaseDispatcher<ExecuteScriptObserver> {

    private static ExecuteScriptDispatcher instance;

    public static synchronized ExecuteScriptDispatcher getInstance() {
        if (instance == null) {
            instance = new ExecuteScriptDispatcher();
        }
        return instance;
    }

    public void onExecuteScriptStarted(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onExecuteScriptStarted(scriptExecutionDTO));
    }

    public void onExecuteScriptSucceeded(ScriptExecutionDTO scriptExecutionDTO, long executionTimeInMilliseconds) {
        trigger(o -> o.onExecuteScriptSucceeded(scriptExecutionDTO, executionTimeInMilliseconds));
    }

    public void onExecuteScriptCancelled(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onExecuteScriptCancelled(scriptExecutionDTO));
    }

    public void onExecuteScriptFailed(ScriptExecutionDTO scriptExecutionDTO , Throwable exception) {
        trigger(o -> o.onExecuteScriptFailed(scriptExecutionDTO, exception));
    }

    public void onExecuteScriptRunning(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onExecuteScriptRunning(scriptExecutionDTO));
    }

    public void onExecuteScriptScheduled(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onExecuteScriptScheduled(scriptExecutionDTO));
    }
}
