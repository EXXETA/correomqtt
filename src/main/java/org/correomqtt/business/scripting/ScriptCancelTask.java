package org.correomqtt.business.scripting;

import org.correomqtt.business.concurrent.NoProgressTask;

public class ScriptCancelTask extends NoProgressTask<Void, Void> {

    private final String executionId;

    public ScriptCancelTask(String executionId) {
        this.executionId = executionId;
    }

    @Override
    protected Void execute() throws Exception {
        ExecutionContextDTO dto = ScriptingBackend.getInstance().getExecutionContextDTO(executionId);
        dto.getContext().close(true);
        return null;
    }
}
