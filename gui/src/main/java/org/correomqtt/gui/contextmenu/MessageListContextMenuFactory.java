package org.correomqtt.gui.contextmenu;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface MessageListContextMenuFactory {
    MessageListContextMenu create(MessageListContextMenuDelegate delegate);

}
