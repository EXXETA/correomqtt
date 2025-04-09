package org.correomqtt.gui.contextmenu;

import javafx.beans.property.BooleanProperty;

public interface DetailContextMenuDelegate extends BaseMessageContextMenuDelegate
{
    BooleanProperty isInlineView();
}
