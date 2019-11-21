package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.gui.model.ConnectionState;

public interface ConnectionViewDelegate {
    void onDisconnect();
    void setTabDirty(String tabId);
    void setConnectionState(String tabId, ConnectionState state);
    void setTabName(String tabId, String name);
}
