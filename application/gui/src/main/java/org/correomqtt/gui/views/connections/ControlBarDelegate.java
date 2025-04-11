package org.correomqtt.gui.views.connections;

import org.correomqtt.gui.model.GuiConnectionState;

public interface ControlBarDelegate {
    void setConnectionState(GuiConnectionState state);
    void saveConnectionUISettings();
    void resetConnectionUISettings();
    void setLayout(boolean publish, boolean subscribe);
}
