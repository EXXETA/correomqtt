package org.correomqtt.gui.views.cell;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;
import org.correomqtt.core.model.Qos;

@AssistedFactory
public interface QosCellFactory {
    QosCell create(ListView<Qos> listView);

}
