package org.correomqtt.gui.controller;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.gui.model.MessagePropertiesDTO;

public interface MessageListViewDelegate {

    void removeMessage(MessageDTO messageDTO);
    void clearMessages();
    void setTabDirty();
    void setUpToForm(MessagePropertiesDTO selectedMessage);
}
