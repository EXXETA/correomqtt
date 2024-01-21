package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface MessageListViewControllerFactory {
    MessageListViewController create(String connectionId,
                                     MessageListViewDelegate delegate);

}
