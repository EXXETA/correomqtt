package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.ExecuteScriptDispatcher;
import org.correomqtt.business.dispatcher.LoadScriptDispatcher;
import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.model.ScriptingDTO;
import org.correomqtt.business.scripting.ScriptingBackend;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;

public class ExecuteScriptService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(ExecuteScriptService.class);
    private final ScriptExecutionDTO scriptExecutionDTO;
    private long executionTimeInMilliseconds = 0;

    public ExecuteScriptService(ScriptExecutionDTO scriptExecutionDTO) {
        this.scriptExecutionDTO = scriptExecutionDTO;
    }

    private String getShortenedCodeForLog() {
        String jsCode = scriptExecutionDTO.getJsCode();
        return jsCode.substring(0, Math.min(20, jsCode.length()));
    }

    public void executeScript() {
        ExecuteScriptDispatcher.getInstance().onExecuteScriptStarted(scriptExecutionDTO);
        LOGGER.info("Start executing Script: {}", getShortenedCodeForLog());
        executionTimeInMilliseconds = ScriptingBackend.executeScript(scriptExecutionDTO);
    }

    @Override
    public void onSucceeded() {
        LOGGER.info("Script execution succeeded: {}", getShortenedCodeForLog());
        ExecuteScriptDispatcher.getInstance().onExecuteScriptSucceeded(scriptExecutionDTO, executionTimeInMilliseconds);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Script execution cancelled: {}", getShortenedCodeForLog());
        ExecuteScriptDispatcher.getInstance().onExecuteScriptCancelled(scriptExecutionDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Script execution failed: {}", getShortenedCodeForLog());
        ExecuteScriptDispatcher.getInstance().onExecuteScriptFailed(scriptExecutionDTO, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Script execution running: {}", getShortenedCodeForLog());
        ExecuteScriptDispatcher.getInstance().onExecuteScriptRunning(scriptExecutionDTO);
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Script execution scheduled: {}", getShortenedCodeForLog());
        ExecuteScriptDispatcher.getInstance().onExecuteScriptScheduled(scriptExecutionDTO);
    }
}

