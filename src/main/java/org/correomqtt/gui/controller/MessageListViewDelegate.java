package org.correomqtt.gui.controller;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.gui.model.MessagePropertiesDTO;

interface MessageListViewDelegate {

    void removeMessage(MessageDTO messageDTO);
    void clearMessages();
    void setTabDirty();
    void setUpToForm(MessagePropertiesDTO selectedMessage);
    void changeFavoriteStatus(MessagePropertiesDTO messageDTO);
}
