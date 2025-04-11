package org.correomqtt.core.scripting;

import lombok.Getter;
import org.correomqtt.di.DefaultBean;
import org.correomqtt.di.Inject;

@Getter
@DefaultBean
public class ScriptTaskFactories {


    private final ScriptSaveTaskFactory saveFactory;
    private final ScriptLoadTaskFactory loadFactory;
    private final ScriptNewTaskFactory newFactory;
    private final ScriptRenameTaskFactory renameFactory;
    private final ScriptDeleteTaskFactory deleteFactory;

    @Inject
    public ScriptTaskFactories(ScriptSaveTaskFactory saveFactory,
                               ScriptLoadTaskFactory loadFactory,
                               ScriptNewTaskFactory newFactory,
                               ScriptRenameTaskFactory renameFactory,
                               ScriptDeleteTaskFactory deleteFactory) {

        this.saveFactory = saveFactory;
        this.loadFactory = loadFactory;
        this.newFactory = newFactory;
        this.renameFactory = renameFactory;
        this.deleteFactory = deleteFactory;
    }
}
