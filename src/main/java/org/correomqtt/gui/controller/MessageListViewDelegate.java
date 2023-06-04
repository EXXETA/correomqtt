package org.correomqtt.gui.controller;

import org.correomqtt.business.model.MessageDTO;
import org.correomqtt.business.model.MessageListViewConfig;
import org.correomqtt.gui.model.MessagePropertiesDTO;

import java.util.function.Supplier;

public interface MessageListViewDelegate {

    void removeMessage(MessageDTO messageDTO);
    void clearMessages();
    void setTabDirty();
    void setUpToForm(MessagePropertiesDTO selectedMessage);

    Supplier<MessageListViewConfig> produceListViewConfig();
}
