package org.correomqtt.core.scripting;

import lombok.Getter;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;

@Getter
@DefaultBean
public class ScriptExecuteTaskFactories {


    private final ScriptCancelTaskFactory cancelFactory;
    private final ScriptExecutionTaskFactory executionFactory;
    private final ScriptLoadLogTaskFactory loadLogFactory;
    private final ScriptDeleteExecutionsTaskFactory deleteExecutionsTask;

    @Inject
    public ScriptExecuteTaskFactories(ScriptCancelTaskFactory cancelFactory,
                                      ScriptExecutionTaskFactory executionFactory,
                                      ScriptLoadLogTaskFactory loadLogFactory,
                                      ScriptDeleteExecutionsTaskFactory deleteExecutionsTask) {

        this.cancelFactory = cancelFactory;
        this.executionFactory = executionFactory;
        this.loadLogFactory = loadLogFactory;
        this.deleteExecutionsTask = deleteExecutionsTask;
    }
}
