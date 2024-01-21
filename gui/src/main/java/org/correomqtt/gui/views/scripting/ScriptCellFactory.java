package org.correomqtt.gui.views.scripting;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;

@AssistedFactory
public interface ScriptCellFactory {
    ScriptCell create(ListView<ScriptFilePropertiesDTO> listView);

}
