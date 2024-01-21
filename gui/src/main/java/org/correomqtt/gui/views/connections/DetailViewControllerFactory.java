package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;
import org.correomqtt.gui.model.MessagePropertiesDTO;

@AssistedFactory
public interface DetailViewControllerFactory {
    DetailViewController create(MessagePropertiesDTO messageDTO,
                                String connectionId,
                                DetailViewDelegate delegate,
                                boolean isInlineView);

}
