package org.correomqtt.gui.controller;

import org.correomqtt.business.model.ConnectionConfigDTO;
import org.correomqtt.gui.model.ConnectionState;

public interface ControlBarDelegate {
    void setConnectionState(ConnectionState state);
    void saveConnectionUISettings();
    void resetConnectionUISettings();
    void setLayout(boolean publish, boolean subscribe);
}
