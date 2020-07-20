package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptExecutionDTO;

public class ScriptCancelDispatcher extends BaseDispatcher<ScriptCancelObserver> {

    private static ScriptCancelDispatcher instance;

    public static synchronized ScriptCancelDispatcher getInstance() {
        if (instance == null) {
            instance = new ScriptCancelDispatcher();
        }
        return instance;
    }

    public void onCancelScriptStarted(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onCancelScriptStarted(scriptExecutionDTO));
    }

    public void onCancelScriptSucceeded(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onCancelScriptSucceeded(scriptExecutionDTO));
    }

    public void onCancelScriptCancelled(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onCancelScriptCancelled(scriptExecutionDTO));
    }

    public void onCancelScriptFailed(ScriptExecutionDTO scriptExecutionDTO , Throwable exception) {
        trigger(o -> o.onCancelScriptFailed(scriptExecutionDTO, exception));
    }

    public void onCancelScriptRunning(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onCancelScriptRunning(scriptExecutionDTO));
    }

    public void onCancelScriptScheduled(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onCancelScriptScheduled(scriptExecutionDTO));
    }
}
