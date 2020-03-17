package org.correomqtt.gui.contextmenu;

import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;

public interface MessageListContextMenuDelegate extends BaseMessageContextMenuDelegate {

    void clearList();

    void removeMessage(MessagePropertiesDTO messageDTO);

    void saveMessage(MessagePropertiesDTO dto);
}
