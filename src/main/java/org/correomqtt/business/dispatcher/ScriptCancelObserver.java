package org.correomqtt.business.dispatcher;

import org.correomqtt.business.model.ScriptExecutionDTO;

public interface ScriptCancelObserver extends BaseObserver {
    void onCancelScriptSucceeded(ScriptExecutionDTO scriptExecutionDTO);

    void onCancelScriptCancelled(ScriptExecutionDTO scriptExecutionDTO);

    void onCancelScriptFailed(ScriptExecutionDTO scriptExecutionDTO, Throwable exception);

    default void onCancelScriptRunning(ScriptExecutionDTO scriptExecutionDTO) {
        // nothing to do
    }

    default void onCancelScriptScheduled(ScriptExecutionDTO scriptExecutionDTOt) {
        // nothing to do
    }

    default void onCancelScriptStarted(ScriptExecutionDTO scriptExecutionDTO) {
        // nothing to do
    }
}
