package org.correomqtt.gui.controller;

import org.correomqtt.gui.model.MessagePropertiesDTO;

public interface SubscriptionViewDelegate {
    void setTabDirty();
    void setUpToForm(MessagePropertiesDTO messageDTO);
}
