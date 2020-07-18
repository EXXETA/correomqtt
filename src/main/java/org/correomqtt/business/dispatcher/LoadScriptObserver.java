package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.ScriptingDTO;

import java.io.File;

public interface LoadScriptObserver extends BaseObserver {
    void onLoadScriptSucceeded(ScriptingDTO scriptingDTO, String scriptCode);

    void onLoadScriptCancelled(ScriptingDTO scriptingDTO);

    void onLoadScriptFailed(ScriptingDTO scriptingDTO, Throwable exception);

    default void onLoadScriptRunning(ScriptingDTO scriptingDTO) {
        // nothing to do
    }

    default void onLoadScriptScheduled(ScriptingDTO scriptingDTO) {
        // nothing to do
    }

    default void onLoadScriptStarted(ScriptingDTO scriptingDTO) {
        // nothing to do
    }
}
