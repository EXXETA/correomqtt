package org.correomqtt.business.services;

import org.correomqtt.business.dispatcher.LoadScriptDispatcher;
import org.correomqtt.business.model.ScriptingDTO;
import org.correomqtt.business.scripting.ScriptingBackend;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class LoadScriptService implements BusinessService {

    private static final Logger LOGGER = LoggerFactory.getLogger(LoadScriptService.class);

    private String scriptCode;
    private final ScriptingDTO scriptingDTO;

    public LoadScriptService(ScriptingDTO scriptingDTO) {
        this.scriptingDTO = scriptingDTO;
    }

    public void loadScript() {
        LoadScriptDispatcher.getInstance().onLoadScriptStarted(scriptingDTO);
        LOGGER.info("Start loading script {}.", scriptingDTO.getPath());
        scriptCode = ScriptingBackend.loadScript(scriptingDTO);
    }

    @Override
    public void onSucceeded() {
        LOGGER.info("Loading script from file {} succeeded", scriptingDTO.getPath().toAbsolutePath());
        LoadScriptDispatcher.getInstance().onLoadScriptSucceeded(scriptingDTO, scriptCode);
    }

    @Override
    public void onCancelled() {
        LOGGER.info("Loading script from file {} cancelled", scriptingDTO.getPath().toAbsolutePath());
        LoadScriptDispatcher.getInstance().onLoadScriptCancelled(scriptingDTO);
    }

    @Override
    public void onFailed(Throwable exception) {
        LOGGER.info("Loading script from file {} failed.", scriptingDTO.getPath().toAbsolutePath());
        LoadScriptDispatcher.getInstance().onLoadScriptFailed(scriptingDTO, exception);
    }

    @Override
    public void onRunning() {
        LOGGER.info("Loading script from file {} running.", scriptingDTO.getPath().toAbsolutePath());
        LoadScriptDispatcher.getInstance().onLoadScriptRunning(scriptingDTO);
    }

    @Override
    public void onScheduled() {
        LOGGER.info("Loading script file {} scheduled.", scriptingDTO.getPath().toAbsolutePath());
        LoadScriptDispatcher.getInstance().onLoadScriptScheduled(scriptingDTO);
    }
}

