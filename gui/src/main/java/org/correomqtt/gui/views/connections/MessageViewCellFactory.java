package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;
import javafx.scene.control.ListView;
import org.correomqtt.core.model.MessageListViewConfig;
import org.correomqtt.gui.model.MessagePropertiesDTO;

import java.util.function.Supplier;

@AssistedFactory
public interface MessageViewCellFactory {
    MessageViewCell create(ListView<MessagePropertiesDTO> listView,
                           Supplier<MessageListViewConfig> listViewConfigGetter);

}
