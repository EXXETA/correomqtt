package org.correomqtt.gui.contextmenu;

import org.correomqtt.gui.model.MessagePropertiesDTO;

public interface BaseMessageContextMenuDelegate extends BaseObjectContextMenuDelegate {
    void showDetailsInSeparateWindow(MessagePropertiesDTO messageDTO);
    void setUpToForm(MessagePropertiesDTO messageDTO);
}
