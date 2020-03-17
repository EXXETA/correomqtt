package org.correomqtt.gui.controller;

import org.correomqtt.gui.model.ConnectionState;

public interface ConnectionViewDelegate {
    void onDisconnect();
    void setTabDirty(String tabId);
    void setConnectionState(String tabId, ConnectionState state);
    void setTabName(String tabId, String name);
}
