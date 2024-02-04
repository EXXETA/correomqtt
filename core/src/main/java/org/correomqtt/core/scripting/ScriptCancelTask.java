package org.correomqtt.core.scripting;

import org.correomqtt.di.Assisted;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;
import org.correomqtt.core.concurrent.SimpleTask;
import org.correomqtt.core.eventbus.EventBus;
import org.correomqtt.core.fileprovider.ScriptingProvider;

@DefaultBean
public class ScriptCancelTask extends SimpleTask {

    private final ScriptingProvider scriptingProvider;
    private final String executionId;

    @Inject
    ScriptCancelTask(ScriptingProvider scriptingProvider,
                            EventBus eventBus,
                            @Assisted String executionId) {
        super(eventBus);
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
