package org.correomqtt.gui.views.cell;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;
import org.correomqtt.gui.model.ConnectionPropertiesDTO;

@AssistedFactory
public interface ConnectionCellFactory {
    ConnectionCell create(ListView<ConnectionPropertiesDTO> listView);

}
