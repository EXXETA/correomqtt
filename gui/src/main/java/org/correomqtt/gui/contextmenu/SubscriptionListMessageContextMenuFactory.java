package org.correomqtt.gui.contextmenu;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface SubscriptionListMessageContextMenuFactory {
    SubscriptionListMessageContextMenu create(SubscriptionListMessageContextMenuDelegate delegate);

}
