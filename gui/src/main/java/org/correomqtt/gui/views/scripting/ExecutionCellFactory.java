package org.correomqtt.gui.views.scripting;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;

@AssistedFactory
public interface ExecutionCellFactory {
    ExecutionCell create(ListView<ExecutionPropertiesDTO> listView);

}
