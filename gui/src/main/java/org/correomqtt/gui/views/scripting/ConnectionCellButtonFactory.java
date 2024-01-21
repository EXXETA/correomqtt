package org.correomqtt.gui.views.scripting;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;

@AssistedFactory
public interface ConnectionCellButtonFactory {
    ConnectionCellButton create(ListView<ConnectionPropertiesDTO> listView);
}
