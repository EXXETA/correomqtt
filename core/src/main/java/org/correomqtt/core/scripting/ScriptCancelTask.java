package org.correomqtt.core.scripting;

import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import dagger.assisted.AssistedInject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.fileprovider.ScriptingProvider;

public class ScriptCancelTask extends SimpleTask {

    private final ScriptingProvider scriptingProvider;
    private final String executionId;

    @AssistedFactory
    public interface Factory {
        ScriptCancelTask create(String executionId);
    }

    @AssistedInject
    public ScriptCancelTask(ScriptingProvider scriptingProvider,
                            @Assisted String executionId) {
        this.scriptingProvider = scriptingProvider;
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
