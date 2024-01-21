package org.correomqtt.gui.views.scripting;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ScriptContextMenuFactory {
    ScriptContextMenu create(ScriptContextMenuDelegate delegate);

}
