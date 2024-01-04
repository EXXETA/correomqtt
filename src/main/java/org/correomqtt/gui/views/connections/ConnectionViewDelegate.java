package org.correomqtt.gui.views.connections;

import org.correomqtt.gui.model.ConnectionState;

public interface ConnectionViewDelegate {

    void onCleanup();
    void onDisconnect();
    void setTabDirty(String tabId);
    void setConnectionState(String tabId, ConnectionState state);
    void setTabName(String tabId, String name);
}
