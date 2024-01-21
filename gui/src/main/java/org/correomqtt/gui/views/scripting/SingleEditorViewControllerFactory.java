package org.correomqtt.gui.views.scripting;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface SingleEditorViewControllerFactory {
    SingleEditorViewController create(SingleEditorViewDelegate delegate,
                                      ScriptFilePropertiesDTO scriptFilePropertiesDTO);

}
