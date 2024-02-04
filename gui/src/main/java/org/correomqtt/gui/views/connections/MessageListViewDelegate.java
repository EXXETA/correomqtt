package org.correomqtt.gui.views.connections;

import org.correomqtt.core.model.MessageDTO;
import org.correomqtt.core.model.MessageListViewConfig;
import org.correomqtt.gui.model.MessagePropertiesDTO;

import java.util.function.Supplier;

public interface MessageListViewDelegate {

    void removeMessage(MessageDTO messageDTO);
    void clearMessages();
    void setTabDirty();
    void setUpToForm(MessagePropertiesDTO selectedMessage);

    Supplier<MessageListViewConfig> produceListViewConfig();
}
