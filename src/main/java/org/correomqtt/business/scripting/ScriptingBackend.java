package org.correomqtt.business.scripting;

import org.correomqtt.business.dispatcher.ScriptResultDispatcher;
import org.correomqtt.business.exception.CorreoMqttExportMessageException;
import org.correomqtt.business.model.ScriptExecutionDTO;
import org.correomqtt.business.model.ScriptingDTO;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Value;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

public class ScriptingBackend {

    private Map<String, ExecutionDTO> executions = new HashMap<>();

    private static ScriptingBackend instance = null;

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

    public void cancelScript(ScriptExecutionDTO scriptExecutionDTO){
        ExecutionDTO execution = executions.get(scriptExecutionDTO.getExecutionId());
        if(execution == null){
            // TODO exception
            return;
        }
        execution.getContext().close(true);
    }

    public void submitScript(ScriptExecutionDTO scriptExecutionDTO) {
        Context context = Context.newBuilder("js")
                .out(scriptExecutionDTO.getOut())
                .build();

        context.getBindings("js").putMember("correo", new CorreoJsBinding(scriptExecutionDTO));

        ExecutionDTO executionDTO = ExecutionDTO.builder()
                .scriptExecutionDTO(scriptExecutionDTO)
                .context(context)
                .startTime(System.currentTimeMillis())
                .build();

        executions.put(scriptExecutionDTO.getExecutionId(), executionDTO);

        CompletableFuture.supplyAsync(() -> context.eval("js", scriptExecutionDTO.getJsCode()))
                .handle((value, t) -> this.onScriptReturned(scriptExecutionDTO.getExecutionId(), value, t));
    }

    private Value onScriptReturned(String executionId, Value value, Throwable t) {
        ExecutionDTO executionDTO = executions.get(executionId);
        long executionTime = System.currentTimeMillis() - executionDTO.getStartTime();
        executionDTO.getContext().close();
        if(t == null){
            ScriptResultDispatcher.getInstance().onScriptExecutionSucceeded(executionDTO.getScriptExecutionDTO(), executionTime);
        }else{
            ScriptResultDispatcher.getInstance().onScriptExecutionFailed(executionDTO.getScriptExecutionDTO(), executionTime, t);
        }
        return null;
    }

}
