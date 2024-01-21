package org.correomqtt.core.scripting;

import lombok.Getter;

import javax.inject.Inject;

@Getter
public class ScriptExecuteTaskFactories {


    private final ScriptCancelTask.Factory cancelFactory;
    private final ScriptExecutionTask.Factory executionFactory;
    private final ScriptLoadLogTask.Factory loadLogFactory;
    private final ScriptDeleteExecutionsTask.Factory deleteExecutionsTask;

    @Inject
    public ScriptExecuteTaskFactories(ScriptCancelTask.Factory cancelFactory,
                                      ScriptExecutionTask.Factory executionFactory,
                                      ScriptLoadLogTask.Factory loadLogFactory,
                                      ScriptDeleteExecutionsTask.Factory deleteExecutionsTask) {

        this.cancelFactory = cancelFactory;
        this.executionFactory = executionFactory;
        this.loadLogFactory = loadLogFactory;
        this.deleteExecutionsTask = deleteExecutionsTask;
    }
}
