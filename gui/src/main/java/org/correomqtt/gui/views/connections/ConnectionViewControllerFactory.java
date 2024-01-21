package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ConnectionViewControllerFactory {
    ConnectionViewController create(String connectionId,
                                    ConnectionViewDelegate delegate);

}
