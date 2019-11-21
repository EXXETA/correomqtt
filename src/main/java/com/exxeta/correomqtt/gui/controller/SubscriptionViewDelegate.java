package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.gui.model.MessagePropertiesDTO;

public interface SubscriptionViewDelegate {
    void setTabDirty();
    void setUpToForm(MessagePropertiesDTO messageDTO);
}
