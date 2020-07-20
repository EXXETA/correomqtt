package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptExecutionDTO;

public class ScriptSubmitDispatcher extends BaseDispatcher<ScriptSubmitObserver> {

    private static ScriptSubmitDispatcher instance;

    public static synchronized ScriptSubmitDispatcher getInstance() {
        if (instance == null) {
            instance = new ScriptSubmitDispatcher();
        }
        return instance;
    }

    public void onSubmitScriptStarted(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onSubmitScriptStarted(scriptExecutionDTO));
    }

    public void onSubmitScriptSucceeded(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onSubmitScriptSucceeded(scriptExecutionDTO));
    }

    public void onSubmitScriptCancelled(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onSubmitScriptCancelled(scriptExecutionDTO));
    }

    public void onSubmitScriptFailed(ScriptExecutionDTO scriptExecutionDTO , Throwable exception) {
        trigger(o -> o.onSubmitScriptFailed(scriptExecutionDTO, exception));
    }

    public void onSubmitScriptRunning(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onSubmitScriptRunning(scriptExecutionDTO));
    }

    public void onSubmitScriptScheduled(ScriptExecutionDTO scriptExecutionDTO) {
        trigger(o -> o.onsubmitScriptScheduled(scriptExecutionDTO));
    }
}
