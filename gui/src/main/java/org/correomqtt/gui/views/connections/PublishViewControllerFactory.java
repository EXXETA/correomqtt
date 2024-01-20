package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface PublishViewControllerFactory {
    PublishViewController create(String connectionId, PublishViewDelegate delegat);

}
