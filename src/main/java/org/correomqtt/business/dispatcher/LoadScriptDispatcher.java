package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptingDTO;

public class LoadScriptDispatcher extends BaseDispatcher<LoadScriptObserver> {

    private static LoadScriptDispatcher instance;

    public static synchronized LoadScriptDispatcher getInstance() {
        if (instance == null) {
            instance = new LoadScriptDispatcher();
        }
        return instance;
    }

    public void onLoadScriptStarted(ScriptingDTO scriptingDTO) {
        trigger(o -> o.onLoadScriptStarted(scriptingDTO));
    }

    public void onLoadScriptSucceeded(ScriptingDTO scriptingDTO, String scriptCode) {
        trigger(o -> o.onLoadScriptSucceeded(scriptingDTO,scriptCode));
    }

    public void onLoadScriptCancelled(ScriptingDTO scriptingDTO) {
        trigger(o -> o.onLoadScriptCancelled(scriptingDTO));
    }

    public void onLoadScriptFailed(ScriptingDTO scriptingDTO , Throwable exception) {
        trigger(o -> o.onLoadScriptFailed(scriptingDTO, exception));
    }

    public void onLoadScriptRunning(ScriptingDTO scriptingDTO) {
        trigger(o -> o.onLoadScriptRunning(scriptingDTO));
    }

    public void onLoadScriptScheduled(ScriptingDTO scriptingDTO) {
        trigger(o -> o.onLoadScriptScheduled(scriptingDTO));
    }
}
