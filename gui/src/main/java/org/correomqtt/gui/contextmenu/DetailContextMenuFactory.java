package org.correomqtt.gui.contextmenu;

import dagger.assisted.AssistedFactory;

@AssistedFactory
public interface DetailContextMenuFactory {
    DetailContextMenu create(DetailContextMenuDelegate delegate);

}
