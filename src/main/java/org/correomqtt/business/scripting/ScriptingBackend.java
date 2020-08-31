package org.correomqtt.business.scripting;

import org.codehaus.plexus.util.StringOutputStream;
import org.correomqtt.business.dispatcher.ScriptResultDispatcher;
import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.model.ScriptingDTO;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.stream.Collectors;

public class ScriptingBackend {

    private static ScriptingBackend instance = null;
    private Logger LOGGER = LoggerFactory.getLogger(ScriptingBackend.class);
    private Map<String, ExecutionDTO> executions = new HashMap<>();

    public static synchronized ScriptingBackend getInstance() {
        if (instance == null) {
            instance = new ScriptingBackend();
            return instance;
        } else {
            return instance;
        }
    }

    public static String loadScript(ScriptingDTO scriptingDTO) {
        StringBuilder codeBuilder = new StringBuilder();
        try {
            Files.lines(scriptingDTO.getPath(), StandardCharsets.UTF_8)
                 .forEach(s -> codeBuilder.append(s).append("\n"));
            return codeBuilder.toString();

        } catch (IOException e) {
            throw new CorreoMqttExportMessageException(e); //TODO
        }
    }

    public void cancelScript(ScriptExecutionDTO scriptExecutionDTO) {
        ExecutionDTO execution = executions.get(scriptExecutionDTO.getExecutionId());
        if (execution == null) {
            // TODO exception
            return;
        }
        execution.getContext().close(true);
    }

    public void submitScript(ScriptExecutionDTO scriptExecutionDTO) {

        LOGGER.debug("Submit script: {}", scriptExecutionDTO.getExecutionId());

        // Important in order to provide permission problems with graalvm
        ExecutorService executorService =  Executors.newSingleThreadExecutor();

        ExecutionDTO executionDTO = ExecutionDTO.builder()
                                                .log(new StringBuilder())
                                                .executorService(executorService)
                                                .scriptExecutionDTO(scriptExecutionDTO)
                                                .startTime(System.currentTimeMillis())
                                                .build();

        ScriptingLogOutputStream out = new ScriptingLogOutputStream(executionDTO);
        executionDTO.setOut(out);

        Context context = Context.newBuilder("js")
                                 .out(out)
                                 .allowAllAccess(true)
                                 .build();
        executionDTO.setContext(context);

        context.getBindings("js").putMember("correo", new CorreoJsBinding(executionDTO));


        executions.put(scriptExecutionDTO.getExecutionId(), executionDTO);


        CompletableFuture.supplyAsync(() -> context.eval("js", scriptExecutionDTO.getJsCode()),executorService)
                         .handle((value, t) -> this.onScriptReturned(scriptExecutionDTO.getExecutionId(), value, t));
    }

    private Value onScriptReturned(String executionId, Value value, Throwable t) {

        if (t == null) {
            LOGGER.debug("Script returned and succeeded: {}", executionId);
        } else {
            LOGGER.error("Script returned and failed: {} \n{}", executionId, t);
        }

        ExecutionDTO executionDTO = executions.get(executionId);
        long executionTime = System.currentTimeMillis() - executionDTO.getStartTime();
        executionDTO.getContext().close();
        ExecutorService executorService = executionDTO.getExecutorService();
        if(!executorService.isShutdown()){
            executorService.shutdown();
        }
        if (t == null) {
            ScriptResultDispatcher.getInstance().onScriptExecutionSucceeded(executionDTO.getScriptExecutionDTO(), executionTime);
        } else {
            ScriptResultDispatcher.getInstance().onScriptExecutionFailed(executionDTO.getScriptExecutionDTO(), executionTime, t);
        }
        return null;
    }

    public List<ExecutionDTO> getExecutions(){
        return new ArrayList<>(executions.values());
    }

}
