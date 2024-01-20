package org.correomqtt.core.scripting;

import org.correomqtt.core.concurrent.SimpleTask;

public class ScriptCancelTask extends SimpleTask {

    private final String executionId;

    public ScriptCancelTask(String executionId) {
        this.executionId = executionId;
    }

    @Override
    protected void execute() {
        ScriptExecutionTask task = ScriptingBackend.getExecutionTask(executionId);
        if (task == null) {
            throw new IllegalStateException("Task for executionId " + executionId + " does not exist.");
        }
        task.cancel();
        ScriptingBackend.removeExecutionTask(executionId);
    }
}
