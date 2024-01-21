package org.correomqtt.core.scripting;

import lombok.Getter;

import javax.inject.Inject;

@Getter
public class ScriptTaskFactories {


    private final ScriptSaveTask.Factory saveFactory;
    private final ScriptLoadTask.Factory loadFactory;
    private final ScriptNewTask.Factory newFactory;
    private final ScriptRenameTask.Factory renameFactory;
    private final ScriptDeleteTask.Factory deleteFactory;

    @Inject
    public ScriptTaskFactories(ScriptSaveTask.Factory saveFactory,
                               ScriptLoadTask.Factory loadFactory,
                               ScriptNewTask.Factory newFactory,
                               ScriptRenameTask.Factory renameFactory,
                               ScriptDeleteTask.Factory deleteFactory) {

        this.saveFactory = saveFactory;
        this.loadFactory = loadFactory;
        this.newFactory = newFactory;
        this.renameFactory = renameFactory;
        this.deleteFactory = deleteFactory;
    }
}
