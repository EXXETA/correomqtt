package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.ScriptCancelDispatcher;
import org.correomqtt.business.dispatcher.ScriptSubmitDispatcher;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.scripting.ScriptingBackend;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class ScriptCancelService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptCancelService.class);
    private final ScriptExecutionDTO scriptExecutionDTO;

    public ScriptCancelService(ScriptExecutionDTO scriptExecutionDTO) {
        this.scriptExecutionDTO = scriptExecutionDTO;
    }

    public void cancelScript() {
        ScriptCancelDispatcher.getInstance().onCancelScriptStarted(scriptExecutionDTO);
        LOGGER.info("Start cancelling script: {}", scriptExecutionDTO.getExecutionId());
    }

    @Override
    public void onSucceeded() {
        LOGGER.info("Script cancellation succeeded: {}", scriptExecutionDTO.getExecutionId());
        ScriptCancelDispatcher.getInstance().onCancelScriptSucceeded(scriptExecutionDTO);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Script cancellation cancelled: {}", scriptExecutionDTO.getExecutionId());
        ScriptCancelDispatcher.getInstance().onCancelScriptCancelled(scriptExecutionDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Script cancellation failed: {}", scriptExecutionDTO.getExecutionId());
        ScriptCancelDispatcher.getInstance().onCancelScriptFailed(scriptExecutionDTO, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Script cancellation running: {}", scriptExecutionDTO.getExecutionId());
        ScriptCancelDispatcher.getInstance().onCancelScriptRunning(scriptExecutionDTO);
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Script cancellation scheduled: {}", scriptExecutionDTO.getExecutionId());
        ScriptCancelDispatcher.getInstance().onCancelScriptScheduled(scriptExecutionDTO);
    }
}

