package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.business.model.MessageDTO;
import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;

public interface MessageListViewDelegate {

    void removeMessage(MessageDTO messageDTO);
    void clearMessages();
    void setTabDirty();
    void setUpToForm(MessagePropertiesDTO selectedMessage);
}
