package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface ControlBarControllerFactory {
    ControlBarController create(String connectionId,
                                    ControlBarDelegate delegate);

}
