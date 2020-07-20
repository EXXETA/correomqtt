package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptExecutionDTO;

public class ScriptResultDispatcher extends BaseDispatcher<ScriptResultObserver> {

    private static ScriptResultDispatcher instance;

    public static synchronized ScriptResultDispatcher getInstance() {
        if (instance == null) {
            instance = new ScriptResultDispatcher();
        }
        return instance;
    }

    public void onScriptExecutionSucceeded(ScriptExecutionDTO scriptExecutionDTO, long executionTimeInMilliseconds) {
        trigger(o -> o.onScriptExecutionSucceeded(scriptExecutionDTO, executionTimeInMilliseconds));
    }

    public void onScriptExecutionFailed(ScriptExecutionDTO scriptExecutionDTO, long executionTimeInMilliseconds, Throwable t) {
        trigger(o -> o.onScriptExecutionFailed(scriptExecutionDTO, executionTimeInMilliseconds, t));
    }

    public void onScriptExecutionTimeout(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onScriptExecutionTimeout(scriptExecutionDTO));
    }

    public void onScriptExecutionCancelled(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onScriptExecutionCancelled(scriptExecutionDTO));
    }

}
