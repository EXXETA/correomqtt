package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;
import org.correomqtt.gui.model.SubscriptionPropertiesDTO;

@AssistedFactory
public interface SubscriptionViewCellFactory {
    SubscriptionViewCell create(ListView<SubscriptionPropertiesDTO> listView);

}
