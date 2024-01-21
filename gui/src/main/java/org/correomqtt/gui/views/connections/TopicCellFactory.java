package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;
import org.correomqtt.gui.model.MessagePropertiesDTO;

@AssistedFactory
public interface TopicCellFactory {
    TopicCell create(ListView<String> listView);

}
