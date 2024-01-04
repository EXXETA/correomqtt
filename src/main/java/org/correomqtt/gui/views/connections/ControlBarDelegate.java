package org.correomqtt.gui.views.connections;

import org.correomqtt.gui.model.ConnectionState;

public interface ControlBarDelegate {
    void setConnectionState(ConnectionState state);
    void saveConnectionUISettings();
    void resetConnectionUISettings();
    void setLayout(boolean publish, boolean subscribe);
}
