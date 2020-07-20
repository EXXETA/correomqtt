package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptExecutionDTO;

public interface ScriptSubmitObserver extends BaseObserver {
    void onSubmitScriptSucceeded(ScriptExecutionDTO scriptExecutionDTO);

    void onSubmitScriptCancelled(ScriptExecutionDTO scriptExecutionDTO);

    void onSubmitScriptFailed(ScriptExecutionDTO scriptExecutionDTO, Throwable exception);

    default void onSubmitScriptRunning(ScriptExecutionDTO scriptExecutionDTO) {
        // nothing to do
    }

    default void onsubmitScriptScheduled(ScriptExecutionDTO scriptExecutionDTOt) {
        // nothing to do
    }

    default void onSubmitScriptStarted(ScriptExecutionDTO scriptExecutionDTO) {
        // nothing to do
    }
}
