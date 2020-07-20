package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.ScriptSubmitDispatcher;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.scripting.ScriptingBackend;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class ScriptSubmitService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ScriptSubmitService.class);
    private final ScriptExecutionDTO scriptExecutionDTO;

    public ScriptSubmitService(ScriptExecutionDTO scriptExecutionDTO) {
        this.scriptExecutionDTO = scriptExecutionDTO;
    }

    public void submitScript() {
        ScriptSubmitDispatcher.getInstance().onSubmitScriptStarted(scriptExecutionDTO);
        LOGGER.info("Start submitting script: {}", scriptExecutionDTO.getExecutionId());
        ScriptingBackend.getInstance().submitScript(scriptExecutionDTO);
    }

    @Override
    public void onSucceeded() {
        LOGGER.info("Script submission succeeded: {}", scriptExecutionDTO.getExecutionId());
        ScriptSubmitDispatcher.getInstance().onSubmitScriptSucceeded(scriptExecutionDTO);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Script submission cancelled: {}", scriptExecutionDTO.getExecutionId());
        ScriptSubmitDispatcher.getInstance().onSubmitScriptCancelled(scriptExecutionDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Script submission failed: {}", scriptExecutionDTO.getExecutionId());
        ScriptSubmitDispatcher.getInstance().onSubmitScriptFailed(scriptExecutionDTO, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Script submission running: {}", scriptExecutionDTO.getExecutionId());
        ScriptSubmitDispatcher.getInstance().onSubmitScriptRunning(scriptExecutionDTO);
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Script submission scheduled: {}", scriptExecutionDTO.getExecutionId());
        ScriptSubmitDispatcher.getInstance().onSubmitScriptScheduled(scriptExecutionDTO);
    }
}

