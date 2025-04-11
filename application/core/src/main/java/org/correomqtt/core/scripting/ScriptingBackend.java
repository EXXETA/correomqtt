package org.correomqtt.core.scripting;

import java.util.List;
import java.util.concurrent.ConcurrentHashMap;

public class ScriptingBackend {

    private static final ConcurrentHashMap<String, ExecutionDTO> EXECUTIONS_DTOS = new ConcurrentHashMap<>();

    private static final ConcurrentHashMap<String, ScriptExecutionTask> EXECUTION_TASKS = new ConcurrentHashMap<>();

    private ScriptingBackend() {
        // private constructor
    }

    public static List<ExecutionDTO> getExecutions() {
        return EXECUTIONS_DTOS.values()
                .stream()
                .toList();
    }

    public static void putExecutionDTO(String executionId, ExecutionDTO dto) {
        EXECUTIONS_DTOS.put(executionId, dto);
    }

    public static ExecutionDTO getExecutionDTO(String executionId) {
        return EXECUTIONS_DTOS.get(executionId);
    }

    public static void removeExecutionsForScript(String filename) {
        EXECUTION_TASKS.entrySet().removeIf(task -> task.getValue().getDto().getScriptFile().getName().equals(filename));
        EXECUTIONS_DTOS.entrySet().removeIf(exec -> exec.getValue().getScriptFile().getName().equals(filename));

    }

    public static ScriptExecutionTask getExecutionTask(String executionId) {
        return EXECUTION_TASKS.get(executionId);
    }

    public static void removeExecutionTask(String executionId) {
        EXECUTION_TASKS.remove(executionId);
    }

    public static void putExecutionTask(ScriptExecutionTask scriptExecutionTask) {
        putExecutionDTO(scriptExecutionTask.getDto().getExecutionId(),scriptExecutionTask.getDto());
        EXECUTION_TASKS.put(scriptExecutionTask.getDto().getExecutionId(), scriptExecutionTask);
    }
}
