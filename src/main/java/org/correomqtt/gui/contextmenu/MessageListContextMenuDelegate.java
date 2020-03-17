package org.correomqtt.gui.contextmenu;

import org.correomqtt.gui.model.MessagePropertiesDTO;

public interface MessageListContextMenuDelegate extends BaseMessageContextMenuDelegate {

    void clearList();

    void removeMessage(MessagePropertiesDTO messageDTO);

    void saveMessage(MessagePropertiesDTO dto);
}
