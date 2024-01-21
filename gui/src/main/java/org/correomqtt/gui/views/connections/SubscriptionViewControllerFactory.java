package org.correomqtt.gui.views.connections;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface SubscriptionViewControllerFactory {
    SubscriptionViewController create(String connectionId, SubscriptionViewDelegate delegate);

}
