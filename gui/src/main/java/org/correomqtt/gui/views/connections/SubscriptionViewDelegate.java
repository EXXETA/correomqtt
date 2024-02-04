package org.correomqtt.gui.views.connections;

import org.correomqtt.gui.model.MessagePropertiesDTO;

public interface SubscriptionViewDelegate {
    void setTabDirty();
    void setUpToForm(MessagePropertiesDTO messageDTO);
}
