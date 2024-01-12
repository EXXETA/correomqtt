package org.correomqtt.business.scripting;

import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

public class ScriptingBackend {
    private static final Map<String, ScriptExecutionTask> EXECUTIONS = new ConcurrentHashMap<>();

    private ScriptingBackend() {
        // private constructor
    }

    public static List<ExecutionDTO> getExecutions() {
        return EXECUTIONS.values()
                .stream()
                .map(ScriptExecutionTask::getDto)
                .toList();
    }

    public static void putExecutionTask(String executionId, ScriptExecutionTask task) {
        EXECUTIONS.put(executionId, task);
    }

    public static ScriptExecutionTask getExecutionTask(String executionId) {
        return EXECUTIONS.get(executionId);
    }

    public static void removeExecutionsForScript(String filename) {
        EXECUTIONS.entrySet().removeIf(exec -> exec.getValue().getDto().getScriptFile().getName().equals(filename));

    }
}
