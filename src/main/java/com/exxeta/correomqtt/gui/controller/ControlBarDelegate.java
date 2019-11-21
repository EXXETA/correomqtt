package com.exxeta.correomqtt.gui.controller;

import com.exxeta.correomqtt.gui.model.ConnectionState;

public interface ControlBarDelegate {
    void setConnectionState(ConnectionState state);

    void setLayout(boolean publish, boolean subscribe);
}
